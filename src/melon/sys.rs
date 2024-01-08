// Abandon hope, all ye who enter here
#![allow(clippy::missing_safety_doc)]

use std::fs::{File, OpenOptions};
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::ptr::drop_in_place;
use std::slice;
use std::sync::{Mutex, MutexGuard, TryLockError};
use std::thread::{spawn, JoinHandle};

use crate::melon::save::write_save;
use crate::utils::localize_pathbuf;
use crate::GAME_TIME;

#[cxx::bridge]
mod sys {
    #[namespace = "Util"]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum MelonFileMode {
        Read = 0b00_00_01,
        Write = 0b00_00_10,
        Preserve = 0b00_01_00,
        NoCreate = 0b00_10_00,
        Text = 0b01_00_00,
    }

    // Replacement stuff

    #[namespace = "Replacements"]
    extern "Rust" {
        #[cxx_name = "EmulatedTime"]
        unsafe fn emulated_time(seconds: *mut i32) -> i32;
    }

    // Util stuff

    #[namespace = "Util"]
    extern "C++" {
        include!("Util.h");

        type MelonFileMode;
        type OpaqueFunction;

        unsafe fn OpaqueFunction_Call(func: *mut OpaqueFunction);
        unsafe fn OpaqueFunction_Free(func: *mut OpaqueFunction);
    }

    // Platform stuff

    // #[namespace = "melonDS::Platform"]
    // extern "C++" {
    //     include!("Platform.h");
    // }

    // interface provided to Platform.cpp, in order to implement Platform.h
    #[namespace = "PlatformImpl"]
    extern "Rust" {
        #[cxx_name = "Thread"]
        #[namespace = "melonDS::Platform"]
        type NdsThread;

        #[cxx_name = "Semaphore"]
        #[namespace = "melonDS::Platform"]
        type NdsSemaphore;

        #[cxx_name = "Mutex"]
        #[namespace = "melonDS::Platform"]
        type NdsMutex;

        #[cxx_name = "FileHandle"]
        #[namespace = "melonDS::Platform"]
        type NdsFileHandle;

        // Instance
        #[cxx_name = "InstanceID"]
        fn instance_id() -> i32;
        #[cxx_name = "InstanceFileSuffix"]
        fn instance_file_suffix() -> String;

        // Camera
        #[cxx_name = "Camera_Start"]
        fn camera_start(num: i32);
        #[cxx_name = "Camera_Stop"]
        fn camera_stop(num: i32);
        #[cxx_name = "Camera_CaptureFrame"]
        unsafe fn camera_capture_frame(
            num: i32,
            frame: *const u32,
            width: i32,
            height: i32,
            yuv: bool,
        );

        // Thread primitive
        #[cxx_name = "Thread_Create"]
        unsafe fn thread_create(func: *mut OpaqueFunction) -> *mut NdsThread;
        #[cxx_name = "Thread_Wait"]
        unsafe fn thread_wait(thread: *mut NdsThread);
        #[cxx_name = "Thread_Free"]
        unsafe fn thread_free(thread: *mut NdsThread);

        // Semaphore primitive
        #[cxx_name = "Semaphore_Create"]
        fn semaphore_create() -> *mut NdsSemaphore;
        #[cxx_name = "Semaphore_Free"]
        unsafe fn semaphore_free(sema: *mut NdsSemaphore);
        #[cxx_name = "Semaphore_Reset"]
        unsafe fn semaphore_reset(sema: *mut NdsSemaphore);
        #[cxx_name = "Semaphore_Wait"]
        unsafe fn semaphore_wait(sema: *mut NdsSemaphore);
        #[cxx_name = "Semaphore_Post"]
        unsafe fn semaphore_post(sema: *mut NdsSemaphore, count: i32);

        // Mutex primitive
        #[cxx_name = "Mutex_Create"]
        fn mutex_create() -> *mut NdsMutex;
        #[cxx_name = "Mutex_Free"]
        unsafe fn mutex_free(mutex: *mut NdsMutex);
        #[cxx_name = "Mutex_Lock"]
        unsafe fn mutex_lock(mutex: *mut NdsMutex);
        #[cxx_name = "Mutex_TryLock"]
        unsafe fn mutex_try_lock(mutex: *mut NdsMutex) -> bool;
        #[cxx_name = "Mutex_Unlock"]
        unsafe fn mutex_unlock(mutex: *mut NdsMutex);

        // LAN
        #[cxx_name = "LAN_Init"]
        fn lan_init() -> bool;
        #[cxx_name = "LAN_DeInit"]
        fn lan_deinit();
        #[cxx_name = "LAN_SendPacket"]
        unsafe fn lan_send_packet(data: *mut u8, len: i32) -> i32;
        #[cxx_name = "LAN_RecvPacket"]
        unsafe fn lan_recv_packet(data: *mut u8) -> i32;

        // multiplayer
        #[cxx_name = "MP_SendAck"]
        unsafe fn mp_send_ack(data: *mut u8, len: i32, timestamp: u64) -> i32;
        #[cxx_name = "MP_SendCmd"]
        unsafe fn mp_send_cmd(data: *mut u8, len: i32, timestamp: u64) -> i32;
        #[cxx_name = "MP_SendReply"]
        unsafe fn mp_send_reply(data: *mut u8, len: i32, timestamp: u64, aid: u16) -> i32;
        #[cxx_name = "MP_SendPacket"]
        unsafe fn mp_send_packet(data: *mut u8, len: i32, timestamp: u64) -> i32;
        #[cxx_name = "MP_RecvPacket"]
        unsafe fn mp_recv_packet(data: *mut u8, timestamp: *mut u64) -> i32;
        #[cxx_name = "MP_RecvHostPacket"]
        unsafe fn mp_recv_host_packet(data: *mut u8, timestamp: *mut u64) -> i32;
        #[cxx_name = "MP_RecvReplies"]
        unsafe fn mp_recv_replies(data: *mut u8, timestamp: u64, aidmask: u16) -> u16;
        #[cxx_name = "MP_Init"]
        fn mp_init() -> bool;
        #[cxx_name = "MP_DeInit"]
        fn mp_deinit();
        #[cxx_name = "MP_Begin"]
        fn mp_begin();
        #[cxx_name = "MP_End"]
        fn mp_end();

        #[cxx_name = "WriteNDSSave"]
        unsafe fn write_nds_save(
            savedata: *const u8,
            savelen: u32,
            writeoffset: u32,
            writelen: u32,
        );

        // File interaction
        #[cxx_name = "OpenFile"]
        fn open_file(path: &CxxString, mode: u8) -> *mut NdsFileHandle;
        #[cxx_name = "OpenLocalFile"]
        fn open_local_file(path: &CxxString, mode: u8) -> *mut NdsFileHandle;

        #[cxx_name = "FileRead"]
        fn file_read();
        #[cxx_name = "FileReadLine"]
        fn file_read_line();
        #[cxx_name = "FileSeek"]
        fn file_seek();
        #[cxx_name = "FileWrite"]
        fn file_write();
        #[cxx_name = "FileWriteFormatted"]
        fn file_write_formatted();

        #[cxx_name = "FileLength"]
        unsafe fn file_length(handle: *mut NdsFileHandle) -> u64;
        #[cxx_name = "IsEndOfFile"]
        // bool IsEndOfFile(FileHandle* file);
        unsafe fn is_end_of_file(handle: *mut NdsFileHandle) -> bool;
        #[cxx_name = "LocalFileExists"]
        fn local_file_exists();
        #[cxx_name = "CloseFile"]
        unsafe fn close_file(handle: *mut NdsFileHandle) -> bool;
        #[cxx_name = "ExportFile"]
        fn export_file();

        // Misc
        #[cxx_name = "SignalStop"]
        fn signal_stop();
        #[cxx_name = "WriteDateTime"]
        fn write_date_time();
        #[cxx_name = "WriteFirmware"]
        fn write_firmware();
        #[cxx_name = "Log"]
        fn log();
    }

    // NDSCart stuff

    #[namespace = "melonDS::NDSCart"]
    unsafe extern "C++" {
        include!("NDSCart.h");

        type CartCommon;
    }

    // NDS stuff

    #[namespace = "melonDS"]
    unsafe extern "C++" {
        include!("NDS.h");

        type NDS;

        // fn Init(&self) -> bool;
        // fn DeInit();
        fn Start(self: Pin<&mut NDS>);
        // fn Stop(self: Pin<&mut NDS>);
        fn Reset(self: Pin<&mut NDS>);

        fn CartInserted(&self) -> bool;

        fn SetKeyMask(self: Pin<&mut NDS>, mask: u32);
        // fn IsLidClosed() -> bool;
        // fn SetLidClosed(closed: bool);

        fn NeedsDirectBoot(&self) -> bool;
        fn SetupDirectBoot(self: Pin<&mut NDS>);

        fn RunFrame(self: Pin<&mut NDS>) -> u32;
    }

    // Shims stuff

    #[namespace = "Shims"]
    unsafe extern "C++" {
        include!("Shims.h");

        pub fn New_NDS() -> UniquePtr<NDS>;

        pub unsafe fn Copy_Framebuffers(nds: &NDS, dest: *mut u8, index: bool) -> bool;
        pub unsafe fn SPU_ReadOutput(nds: Pin<&mut NDS>, data: *mut i16, samples: i32) -> i32;

        pub unsafe fn ReadSavestate(nds: Pin<&mut NDS>, contents: *const u8, len: i32) -> bool;
        pub unsafe fn WriteSavestate(nds: Pin<&mut NDS>, data: *mut CxxVector<u8>) -> bool;

        pub unsafe fn CurrentFrame(nds: &NDS) -> u32;

        pub unsafe fn MainRAM(nds: &NDS) -> *const u8;
        pub unsafe fn MainRAMMut(nds: Pin<&mut NDS>) -> *mut u8;
        pub unsafe fn MainRAMMaxSize(nds: &NDS) -> u32;

        pub unsafe fn NDS_SetNDSCart(nds: Pin<&mut NDS>, cart: UniquePtr<CartCommon>);

        pub unsafe fn ParseROMWithSave(
            romdata: *const u8,
            romlen: u32,
            savedata: *const u8,
            savelen: u32,
        ) -> UniquePtr<CartCommon>;
    }

    // Implementations on types

    unsafe impl UniquePtr<CartCommon> {}
    unsafe impl UniquePtr<NDS> {}
}

fn instance_id() -> i32 {
    0
}

fn instance_file_suffix() -> String {
    String::from(".instance")
}

fn camera_start(num: i32) {}

fn camera_stop(num: i32) {}

unsafe fn camera_capture_frame(num: i32, frame: *const u32, width: i32, height: i32, yuv: bool) {}

struct NdsThread {
    inner: Option<std::thread::JoinHandle<()>>,
}

impl NdsThread {
    pub fn new(func: JoinHandle<()>) -> Self {
        Self { inner: Some(func) }
    }

    unsafe fn wait(this: *mut Self) {
        (*this).inner.take().unwrap().join().unwrap();
    }
}

struct OpaqueWrapper(*mut OpaqueFunction);

impl OpaqueWrapper {
    pub unsafe fn run(&mut self) {
        OpaqueFunction_Call(self.0);
    }
}

impl Drop for OpaqueWrapper {
    fn drop(&mut self) {
        unsafe {
            OpaqueFunction_Free(self.0);
        }
    }
}

unsafe impl Send for OpaqueWrapper {}
unsafe impl Sync for OpaqueWrapper {}

unsafe fn thread_create(func: *mut OpaqueFunction) -> *mut NdsThread {
    let mut wrapper = OpaqueWrapper(func);
    let nds_thread = Box::new(NdsThread::new(spawn(move || {
        wrapper.run();
    })));
    Box::leak(nds_thread)
}
unsafe fn thread_wait(thread: *mut NdsThread) {
    NdsThread::wait(thread);
}
unsafe fn thread_free(thread: *mut NdsThread) {
    drop_in_place(thread);
}

struct NdsSemaphore {
    capacity: Mutex<usize>,
}

impl NdsSemaphore {
    pub fn new() -> Self {
        Self {
            capacity: Mutex::new(0),
        }
    }
}

fn semaphore_create() -> *mut NdsSemaphore {
    let sema = Box::new(NdsSemaphore::new());
    Box::leak(sema)
}
// drops the semaphore
unsafe fn semaphore_free(sema: *mut NdsSemaphore) {
    drop_in_place(sema);
}
// acquire all available resources
unsafe fn semaphore_reset(sema: *mut NdsSemaphore) {
    *(*sema).capacity.get_mut().unwrap() = 0;
}
// get one resource
unsafe fn semaphore_wait(sema: *mut NdsSemaphore) {
    loop {
        let available = (*sema).capacity.get_mut().unwrap();
        if *available > 0 {
            *available -= 1;
            break;
        }
    }
}
// release a certain number of resources
// memory stuff is managed in cpp. Just add them I guess
unsafe fn semaphore_post(sema: *mut NdsSemaphore, count: i32) {
    let cap = (*sema).capacity.get_mut().unwrap();
    *cap += count as usize;
}

// probably invoking some 8th cardinal sin
struct NdsMutex {
    mutex: Mutex<()>,
    guard: Option<MutexGuard<'static, ()>>,
}

impl NdsMutex {
    pub fn new() -> NdsMutex {
        Self {
            mutex: Mutex::new(()),
            guard: None,
        }
    }

    pub unsafe fn lock(this: *mut Self) {
        let guard = (*this).mutex.lock().unwrap();
        (*this).guard = Some(guard);
    }

    pub unsafe fn try_lock(this: *mut Self) -> Result<(), TryLockError<MutexGuard<'static, ()>>> {
        let guard = (*this).mutex.try_lock()?;
        (*this).guard = Some(guard);
        Ok(())
    }

    pub unsafe fn unlock(this: *mut Self) {
        (*this).guard.take();
    }
}

fn mutex_create() -> *mut NdsMutex {
    let mutex = Box::new(NdsMutex::new());
    Box::leak(mutex)
}
unsafe fn mutex_lock(mutex: *mut NdsMutex) {
    NdsMutex::lock(mutex);
}
unsafe fn mutex_try_lock(mutex: *mut NdsMutex) -> bool {
    NdsMutex::try_lock(mutex).is_ok()
}
unsafe fn mutex_unlock(mutex: *mut NdsMutex) {
    NdsMutex::unlock(mutex);
}
unsafe fn mutex_free(mutex: *mut NdsMutex) {
    drop_in_place(mutex);
}

struct NdsFileHandle {
    handle: File,
    cursor: Cursor<Vec<u8>>,
    read: bool,
    write: bool,
}

impl NdsFileHandle {
    pub fn new(path: String, read: bool, write: bool, create: bool, truncate: bool) -> Self {
        // get a copy of the file contents, after create and truncate side effects
        // read and write are true here to accomplish this
        let mut handle = OpenOptions::new()
            .read(true)
            .write(true)
            .create(write && create)
            .truncate(write && truncate)
            .open(path.to_string())
            .unwrap();

        let mut contents = vec![];
        handle.read_to_end(&mut contents).unwrap();

        Self {
            handle,
            cursor: Cursor::new(contents),
            read,
            write,
        }
    }

    pub fn length(&self) -> u64 {
        let current = self.cursor.position();
        let end = self.cursor.seek(SeekFrom::End(0)).unwrap();
        self.cursor.set_position(current);
        end
    }

    pub fn is_end(&self) -> bool {
        let pos = self.cursor.position();
        pos == self.length()
    }
}

fn open_file_rust(path: String, mode: u8) -> *mut NdsFileHandle {
    let read = (mode & MelonFileMode::Read.repr) > 0;
    let write = (mode & MelonFileMode::Write.repr) > 0;
    let create = (mode & MelonFileMode::NoCreate.repr) == 0;
    let truncate = (mode & MelonFileMode::Preserve.repr) == 0;

    let file_handle = Box::new(NdsFileHandle::new(path, read, write, create, truncate));

    Box::leak(file_handle)
}

fn open_file(path: &CxxString, mode: u8) -> *mut NdsFileHandle {
    open_file_rust(path.to_string(), mode)
}

fn open_local_file(path: &CxxString, mode: u8) -> *mut NdsFileHandle {
    let local_path = localize_pathbuf(path.to_string()).to_string_lossy();
    open_file_rust(local_path.into_owned(), mode)
}

unsafe fn file_read(data: *mut u8, size: u64, count: u64, handle: *mut NdsFileHandle) {
    // let src = 
    // data.copy_from(, count)
}
fn file_read_line() {}
fn file_seek() {}
fn file_write() {}
fn file_write_formatted() {}
unsafe fn file_length(handle: *mut NdsFileHandle) -> u64 {
    (*handle).length()
}
unsafe fn is_end_of_file(handle: *mut NdsFileHandle) -> bool {
    (*handle).is_end()
}
fn local_file_exists() {}
unsafe fn close_file(handle: *mut NdsFileHandle) -> bool {
    drop_in_place(handle);
    true
}
fn export_file() {}

fn signal_stop() {}
fn write_date_time() {}
fn write_firmware() {}
fn log() {}

fn lan_init() -> bool {
    // TODO: provide an event subscription
    false
}

unsafe fn lan_send_packet(data: *mut u8, len: i32) -> i32 {
    0
}
unsafe fn lan_recv_packet(data: *mut u8) -> i32 {
    0
}

fn lan_deinit() {}

unsafe fn mp_send_ack(data: *mut u8, len: i32, timestamp: u64) -> i32 {
    0
}
unsafe fn mp_send_cmd(data: *mut u8, len: i32, timestamp: u64) -> i32 {
    0
}
unsafe fn mp_send_reply(data: *mut u8, len: i32, timestamp: u64, aid: u16) -> i32 {
    0
}
unsafe fn mp_send_packet(data: *mut u8, len: i32, timestamp: u64) -> i32 {
    0
}
unsafe fn mp_recv_packet(data: *mut u8, timestamp: *mut u64) -> i32 {
    0
}
unsafe fn mp_recv_host_packet(data: *mut u8, timestamp: *mut u64) -> i32 {
    0
}
unsafe fn mp_recv_replies(data: *mut u8, timestamp: u64, aidmask: u16) -> u16 {
    0
}
fn mp_init() -> bool {
    true
}
fn mp_deinit() {}
fn mp_begin() {}
fn mp_end() {}

unsafe fn write_nds_save(savedata: *const u8, savelen: u32, writeoffset: u32, writelen: u32) {
    write_save(
        slice::from_raw_parts(savedata, savelen as usize),
        writeoffset as usize,
        writelen as usize,
    );
}

unsafe fn emulated_time(seconds: *mut i32) -> i32 {
    let now = GAME_TIME.lock().unwrap().timestamp() as i32;
    if !seconds.is_null() {
        *seconds = now;
        0
    } else {
        now
    }
}

use cxx::CxxString;
pub use sys::*;
