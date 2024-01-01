use std::{
    ptr::drop_in_place,
    slice,
    sync::{Mutex, MutexGuard, TryLockError},
    thread::{spawn, JoinHandle},
};

use crate::melon::{save::write_save, subscriptions};

#[cxx::bridge]
pub mod sys {
    #[namespace = "Glue"]
    extern "Rust" {
        #[cxx_name = "Thread"]
        #[namespace = "Platform"]
        type NdsThread;

        #[cxx_name = "Semaphore"]
        #[namespace = "Platform"]
        type NdsSemaphore;

        #[cxx_name = "Mutex"]
        #[namespace = "Platform"]
        type NdsMutex;

        #[cxx_name = "InstanceID"]
        fn instance_id() -> i32;
        #[cxx_name = "InstanceFileSuffix"]
        fn instance_file_suffix() -> String;

        #[cxx_name = "StopEmu"]
        fn stop_emu();

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

        #[cxx_name = "Thread_Create"]
        unsafe fn thread_create(func: *mut OpaqueFunction) -> *mut NdsThread;
        #[cxx_name = "Thread_Wait"]
        unsafe fn thread_wait(thread: *mut NdsThread);
        #[cxx_name = "Thread_Free"]
        unsafe fn thread_free(thread: *mut NdsThread);

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
    }

    #[namespace = "Util"]
    unsafe extern "C++" {
        include!("Util.h");

        type OpaqueFunction;

        unsafe fn OpaqueFunction_Call(func: *mut OpaqueFunction);
        unsafe fn OpaqueFunction_Free(func: *mut OpaqueFunction);
    }
}

fn instance_id() -> i32 {
    // std::process::id() as i32
    0
}

fn instance_file_suffix() -> String {
    format!("{}", instance_id())
}

fn stop_emu() {
    subscriptions::STOP_EMU.lock().unwrap().call(())
}

fn camera_start(num: i32) {}

fn camera_stop(num: i32) {}

unsafe fn camera_capture_frame(
    num: i32,
    frame: *const u32,
    width: i32,
    height: i32,
    yuv: bool,
) {
}

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

use crate::melon::sys::platform::{
    OpaqueFunction, OpaqueFunction_Call, OpaqueFunction_Free,
};

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

    pub unsafe fn try_lock(
        this: *mut Self,
    ) -> Result<(), TryLockError<MutexGuard<'static, ()>>> {
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

fn lan_deinit() {
    subscriptions::STOP_EMU.lock().unwrap().call(())
}

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

pub use sys::*;
