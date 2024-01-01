#[cxx::bridge(namespace = "melonDS::NDSCart")]
mod sys {
    unsafe extern "C++" {
        include!("NDSCart.h");

        type CartCommon;
    }

    unsafe impl UniquePtr<CartCommon> {}
}

pub use sys::*;
