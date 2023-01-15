#[cxx::bridge]
mod ffi {
    extern "C++" {
        // #[cxx_name = "FILE"]
        // type CFile;
    }
}

mod glue {
    use std::path::PathBuf;

    #[cxx::bridge(namespace = "Rust")]
    mod rust {
        extern "Rust" {
            #[cxx_name = "LocalizePath"]
            fn localize_path(path: String) -> String;
        }
    }

    fn localize_path(path: String) -> String {
        let pathbuf = PathBuf::from(path);
        if pathbuf.is_absolute() {
            pathbuf.to_string_lossy().into()
        } else {
            std::env::current_exe()
                .expect("Couldn't get target executable path")
                .parent()
                .expect("Failed to get path to current executable's parent folder")
                .join("melon")
                .to_string_lossy()
                .into()
        }
    }
}

#[cxx::bridge(namespace = "NDS")]
pub mod nds {
    unsafe extern "C++" {
        include!("NDS.h");
        include!("GPU.h");

        fn Init() -> bool;
        fn DeInit();

        fn SetConsoleType(console_type: i32);

        fn CartInserted() -> bool;
        unsafe fn LoadCart(
            romdata: *const u8,
            romlen: u32,
            savedata: *const u8,
            savelen: u32,
        ) -> bool;

        fn Start();
        fn Stop();
        fn RunFrame() -> u32;
    }
}

#[cxx::bridge(namespace = "GPU")]
pub mod gpu {
    unsafe extern "C++" {
        include!("GPU.h");

        fn InitRenderer(renderer: i32);
    }
}

#[allow(unused_variables)]
pub mod platform {
    use std::{
        ptr::drop_in_place,
        sync::{Mutex, MutexGuard},
    };

    use crate::melon::subscriptions;

    #[cxx::bridge(namespace = "Glue")]
    pub mod glue {
        extern "Rust" {
            #[cxx_name = "Mutex"]
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

            #[cxx_name = "GetConfigBool"]
            fn get_config_bool(entry: ConfigEntry) -> bool;
            #[cxx_name = "GetConfigInt"]
            fn get_config_int(entry: ConfigEntry) -> i32;
            #[cxx_name = "GetConfigString"]
            fn get_config_string(entry: ConfigEntry) -> String;

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

        #[repr(u32)]
        #[derive(Debug, Clone, Copy)]
        enum ConfigEntry {
            // JIT_Enable,
            // JIT_MaxBlockSize,
            // JIT_LiteralOptimizations,
            // JIT_BranchOptimizations,
            // JIT_FastMemory,
            ExternalBIOSEnable,
            BIOS9Path,
            BIOS7Path,
            FirmwarePath,
            DSi_BIOS9Path,
            DSi_BIOS7Path,
            DSi_FirmwarePath,
            DSi_NANDPath,
            DLDI_Enable,
            DLDI_ImagePath,
            DLDI_ImageSize,
            DLDI_ReadOnly,
            DLDI_FolderSync,
            DLDI_FolderPath,
            DSiSD_Enable,
            DSiSD_ImagePath,
            DSiSD_ImageSize,
            DSiSD_ReadOnly,
            DSiSD_FolderSync,
            DSiSD_FolderPath,
            Firm_OverrideSettings,
            Firm_Username,
            Firm_Language,
            Firm_BirthdayMonth,
            Firm_BirthdayDay,
            Firm_Color,
            Firm_Message,
            Firm_MAC,
            AudioBitrate,
        }
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

    fn instance_file_suffix() -> String {
        format!("{}", instance_id())
    }

    fn stop_emu() {
        subscriptions::STOP_EMU.lock().unwrap().call(())
    }

    fn camera_start(num: i32) {}

    fn camera_stop(num: i32) {}

    use crate::config;
    use glue::ConfigEntry;
    fn get_config_bool(entry: ConfigEntry) -> bool {
        match entry {
            ConfigEntry::ExternalBIOSEnable => config::EXTERNAL_BIOSENABLE,
            ConfigEntry::DLDI_Enable => config::DLDIENABLE,
            ConfigEntry::DLDI_ReadOnly => config::DLDIREAD_ONLY,
            ConfigEntry::DLDI_FolderSync => config::DLDIFOLDER_SYNC,
            ConfigEntry::DSiSD_Enable => config::DSI_SDENABLE,
            ConfigEntry::DSiSD_ReadOnly => config::DSI_SDREAD_ONLY,
            ConfigEntry::DSiSD_FolderSync => config::DSI_SDFOLDER_SYNC,
            ConfigEntry::Firm_OverrideSettings => config::FIRMWARE_OVERRIDE_SETTINGS,
            _ => false,
        }
    }
    fn get_config_int(entry: ConfigEntry) -> i32 {
        let img_sizes = [0, 256, 512, 1024, 2048, 4096];
        match entry {
            ConfigEntry::DLDI_ImageSize => config::DLDISIZE,
            ConfigEntry::DSiSD_ImageSize => config::DSI_SDSIZE,
            ConfigEntry::Firm_Language => config::FIRMWARE_LANGUAGE,
            ConfigEntry::Firm_BirthdayMonth => config::FIRMWARE_BIRTHDAY_MONTH,
            ConfigEntry::Firm_BirthdayDay => config::FIRMWARE_BIRTHDAY_DAY,
            ConfigEntry::Firm_Color => config::FIRMWARE_FAVOURITE_COLOUR,
            ConfigEntry::AudioBitrate => config::AUDIO_BITRATE,
            _ => 0,
        }
    }
    fn get_config_string(entry: ConfigEntry) -> String {
        match entry {
            ConfigEntry::BIOS9Path => config::BIOS9_PATH,
            ConfigEntry::BIOS7Path => config::BIOS7_PATH,
            ConfigEntry::FirmwarePath => config::FIRMWARE_PATH,
            ConfigEntry::DSi_BIOS9Path => config::DSI_BIOS9_PATH,
            ConfigEntry::DSi_BIOS7Path => config::DSI_BIOS7_PATH,
            ConfigEntry::DSi_FirmwarePath => config::DSI_FIRMWARE_PATH,
            ConfigEntry::DSi_NANDPath => config::DSI_NANDPATH,
            ConfigEntry::DLDI_ImagePath => config::DLDISDPATH,
            ConfigEntry::DLDI_FolderPath => config::DLDIFOLDER_PATH,
            ConfigEntry::DSiSD_ImagePath => config::DSI_SDPATH,
            ConfigEntry::DSiSD_FolderPath => config::DSI_SDFOLDER_PATH,
            ConfigEntry::Firm_Username => config::FIRMWARE_USERNAME,
            ConfigEntry::Firm_Message => config::FIRMWARE_MESSAGE,
            _ => "",
        }
        .into()
    }

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
    fn mp_init() -> bool {
        true
    }
    fn mp_deinit() {}
    fn mp_begin() {}
    fn mp_end() {}

    unsafe fn write_nds_save(savedata: *const u8, savelen: u32, writeoffset: u32, writelen: u32) {
        // TODO
    }
}

pub use ffi::*;
