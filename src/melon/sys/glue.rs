use crate::utils::localize_pathbuf;

#[cxx::bridge(namespace = "Rust")]
mod sys {
    extern "Rust" {
        #[cxx_name = "LocalizePath"]
        fn localize_path(path: String) -> String;
    }
}

pub fn localize_path(path: String) -> String {
    localize_pathbuf(path).to_string_lossy().into()
}

pub use sys::*;
