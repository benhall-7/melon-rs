// Abandon hope, all ye who enter here
#![allow(clippy::missing_safety_doc)]

// #[cxx::bridge(namespace = "SPU")]
// pub mod spu {
//     unsafe extern "C++" {
//         include!("SPU.h");

//         /// NOTE: the data length must be greater than twice the sample count
//         unsafe fn ReadOutput(data: *mut i16, samples: i32) -> i32;
//     }
// }

pub mod glue;
pub mod platform;
pub mod shims;
pub mod replacements;

pub mod nds;
pub mod nds_cart;