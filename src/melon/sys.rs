use autocxx::prelude::*;

include_cpp! {
    #include "NDS.h"
    #include "Savestate.h"
    #include "types.h"
    safety!(unsafe)

    generate!("Savestate")

    generate!("NDS::Init")
    generate!("NDS::DeInit")
    generate!("NDS::Reset")
    generate!("NDS::Start")
    generate!("NDS::Stop")
    generate!("NDS::DoSavestate")
    generate!("NDS::SetARM9RegionTimings")
    generate!("NDS::SetARM7RegionTimings")
    generate!("NDS::SetConsoleType")
    generate!("NDS::LoadBIOS")
    generate!("NDS::LoadCart")
    generate!("NDS::LoadSave")
    generate!("NDS::EjectCart")
    generate!("NDS::CartInserted")
}

pub use ffi::*;
