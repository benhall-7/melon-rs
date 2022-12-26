#[cxx::bridge]
mod ffi {
    extern "C++" {
        include!("Savestate.h");
        include!("types.h");

        type Savestate;
    }
}

#[cxx::bridge(namespace = "NDS")]
pub mod nds {
    unsafe extern "C++" {
        include!("NDS.h");
        include!("types.h");

        fn Init() -> bool;
        fn DeInit();
        fn SetConsoleType(console_type: i32);
        fn CartInserted() -> bool;

        // generate!("NDS::Reset")
        // generate!("NDS::Start")
        // generate!("NDS::Stop")
        // generate!("NDS::DoSavestate")
        // generate!("NDS::SetARM9RegionTimings")
        // generate!("NDS::SetARM7RegionTimings")
        // // this just calls ::Reset
        // // generate!("NDS::LoadBIOS")
        // generate!("NDS::LoadCart")
        // generate!("NDS::LoadSave")
        // generate!("NDS::EjectCart")
    }
}

#[allow(unused_variables)]
pub mod platform {
    use std::{
        ptr::drop_in_place,
        sync::{Mutex, MutexGuard},
    };

    use crate::melon::subscriptions;

    #[cxx::bridge(namespace = "Platform")]
    pub mod PlatformHeader {
        extern "Rust" {
            #[cxx_name = "Mutex"]
            type NdsMutex;

            #[cxx_name = "InstanceID"]
            fn instance_id() -> i32;

            #[cxx_name = "StopEmu"]
            fn stop_emu();

            #[cxx_name = "Camera_Start"]
            fn camera_start(num: i32);
            #[cxx_name = "Camera_Stop"]
            fn camera_stop(num: i32);

            #[cxx_name = "Mutex_Create"]
            fn mutex_create() -> *mut NdsMutex;
            #[cxx_name = "Mutex_Free"]
            unsafe fn mutex_free(mutex: *mut NdsMutex);
            #[cxx_name = "Mutex_Lock"]
            unsafe fn mutex_lock(mutex: *mut NdsMutex);
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
            unsafe fn mp_send_ack(data: *mut u8, len: i32, timestamp: u64);
            #[cxx_name = "MP_SendCmd"]
            unsafe fn mp_send_cmd(data: *mut u8, len: i32, timestamp: u64);
            #[cxx_name = "MP_SendReply"]
            unsafe fn mp_send_reply(data: *mut u8, len: i32, timestamp: u64, aid: u16);
            #[cxx_name = "MP_SendPacket"]
            unsafe fn mp_send_packet(data: *mut u8, len: i32, timestamp: u64);
            #[cxx_name = "MP_RecvPacket"]
            unsafe fn mp_recv_packet(data: *mut u8, timestamp: *mut u64);
            #[cxx_name = "MP_RecvHostPacket"]
            unsafe fn mp_recv_host_packet(data: *mut u8, timestamp: *mut u64);
            #[cxx_name = "MP_RecvReplies"]
            unsafe fn mp_recv_replies(data: *mut u8, timestamp: u64, aidmask: u16) -> u16;
            #[cxx_name = "MP_Init"]
            fn mp_init();
            #[cxx_name = "MP_DeInit"]
            fn mp_deinit();
            #[cxx_name = "MP_Begin"]
            fn mp_begin();
            #[cxx_name = "MP_End"]
            fn mp_end();
        }

        // extern "C++" {
        //     include!("Platform.h");
        //     type ConfigEntry;
        // }
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

        pub unsafe fn unlock(this: *mut Self) {
            (*this).guard.take();
        }
    }

    fn instance_id() -> i32 {
        std::process::id() as i32
    }

    fn stop_emu() {
        subscriptions::STOP_EMU.lock().unwrap().call(())
    }

    fn camera_start(num: i32) {}

    fn camera_stop(num: i32) {}

    fn mutex_create() -> *mut NdsMutex {
        let mutex = Box::new(NdsMutex::new());
        Box::leak(mutex)
    }
    unsafe fn mutex_lock(mutex: *mut NdsMutex) {
        NdsMutex::lock(mutex);
    }
    unsafe fn mutex_unlock(mutex: *mut NdsMutex) {
        NdsMutex::unlock(mutex);
    }
    unsafe fn mutex_free(mutex: *mut NdsMutex) {
        drop_in_place(mutex);
    }

    fn lan_init() -> bool {
        // TODO: provide an event subscription
        true
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

    unsafe fn mp_send_ack(data: *mut u8, len: i32, timestamp: u64) {}
    unsafe fn mp_send_cmd(data: *mut u8, len: i32, timestamp: u64) {}
    unsafe fn mp_send_reply(data: *mut u8, len: i32, timestamp: u64, aid: u16) {}
    unsafe fn mp_send_packet(data: *mut u8, len: i32, timestamp: u64) {}
    unsafe fn mp_recv_packet(data: *mut u8, timestamp: *mut u64) {}
    unsafe fn mp_recv_host_packet(data: *mut u8, timestamp: *mut u64) {}
    unsafe fn mp_recv_replies(data: *mut u8, timestamp: u64, aidmask: u16) -> u16 {
        0
    }
    fn mp_init() {}
    fn mp_deinit() {}
    fn mp_begin() {}
    fn mp_end() {}
}

pub use ffi::*;
