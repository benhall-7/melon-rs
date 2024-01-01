use cxx::memory::UniquePtrTarget;

#[cxx::bridge(namespace = "melonDS")]
mod sys {

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

    unsafe impl UniquePtr<NDS> {}
}

pub use sys::*;
