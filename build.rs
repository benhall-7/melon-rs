use std::env;
use std::path::PathBuf;

use cmake::Config;

fn main() {
    // build melonDS
    let dst = Config::new("melonDS")
        .define("CMAKE_PREFIX_PATH", "/usr/local/opt/qt;/usr/local/opt/libarchive")
        .define("USE_QT6", "ON")
        .build_target("core")
        .build();
    
    // Check output of `cargo build --verbose`, should see something like:
    // -L native=/path/runng/target/debug/build/runng-sys-abc1234/out
    // That contains output from cmake
    println!("cargo:rustc-link-search=native={}", dst.display());
    // // Tell rustc to use nng static library
    // println!("cargo:rustc-link-lib=static=melonDS");
}
