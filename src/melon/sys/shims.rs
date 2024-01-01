
#[cxx::bridge(namespace = "Shims")]
mod sys {
    unsafe extern "C++" {
        include!("Shims.h");

        // I think this is a workaround to a bug.
        // CXX is trying to produce "Shims::NDS" even though it should link to the real NDS.
        // Here, I set it explicitly to the correct namespace
        #[namespace = "melonDS"]
        type NDS = crate::melon::sys::nds::NDS;
        #[namespace = "melonDS::NDSCart"]
        type CartCommon = crate::melon::sys::nds_cart::CartCommon;

        pub fn New_NDS() -> UniquePtr<NDS>;

        pub unsafe fn ReadSavestate(nds: Pin<&mut NDS>, contents: *const u8, len: i32) -> bool;
        pub unsafe fn WriteSavestate(
            nds: Pin<&mut NDS>,
            store: unsafe fn(source: *const u8, len: i32),
        ) -> bool;

        pub unsafe fn Copy_Framebuffers(nds: Pin<&NDS>, dest: *mut u8, index: bool) -> bool;
        pub unsafe fn CurrentFrame(nds: Pin<&NDS>) -> u32;

        pub unsafe fn MainRAM(nds: Pin<&NDS>) -> *const u8;
        pub unsafe fn MainRAMMut(nds: Pin<&mut NDS>) -> *mut u8;
        pub unsafe fn MainRAMMaxSize(nds: Pin<&NDS>) -> u32;
    
        pub unsafe fn NDS_SetNDSCart(nds: Pin<&mut NDS>, cart: UniquePtr<CartCommon>);

        pub unsafe fn ParseROMWithSave(
            romdata: *const u8,
            romlen: u32,
            savedata: *const u8,
            savelen: u32,
        ) -> UniquePtr<CartCommon>;
    }
}

pub use sys::*;
