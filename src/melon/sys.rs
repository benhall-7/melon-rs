#[cxx::bridge]
mod ffi {
    extern "C++" {
        include!("Savestate.h");
        include!("types.h");

        type Savestate;
    }
}

#[cxx::bridge(namespace = "NDS")]
pub mod NDS {
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

pub mod Platform {
    #[cxx::bridge(namespace = "Platform")]
    pub mod PlatformHeader {
        // type ConfigEntry;

        extern "Rust" {
            #[cxx_name = "InstanceID"]
            fn instance_id() -> i32;
        }

        // extern "C++" {
        //     include!("Platform.h");
        //     type ConfigEntry;
        // }
    }

    fn instance_id() -> i32 {
        std::process::id() as i32
    }
}

pub use ffi::*;
