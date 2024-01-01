use crate::GAME_TIME;

#[cxx::bridge(namespace = "Replacements")]
pub mod externs {
    extern "Rust" {
        #[cxx_name = "EmulatedTime"]
        unsafe fn emulated_time(seconds: *mut i32) -> i32;
    }
}

unsafe fn emulated_time(seconds: *mut i32) -> i32 {
    let now = GAME_TIME.lock().unwrap().timestamp() as i32;
    if !seconds.is_null() {
        *seconds = now;
        0
    } else {
        now
    }
}