use std::env::current_dir;

use cmake::Config;

fn main() {
    println!("cargo:rerun-if-changed=src/melon/sys.rs");
    println!("cargo:rerun-if-changed=melonDS/");
    println!("cargo:rerun-if-changed=src/melon/cpp/");

    let directory = current_dir().unwrap();
    let replacements = directory.join("src/melon/cpp/Replacements.h");

    // build melonDS
    let dst = Config::new("melonDS")
        .define("BUILD_QT_SDL", "OFF")
        .define("ENABLE_JIT", "OFF")
        .define("ENABLE_GDBSTUB", "OFF")
        .build_target("all")
        .define("INTERCEPT", "ON")
        // this lets us replace the in-game time invocation with an emulated version
        .define(
            "CMAKE_CXX_FLAGS",
            format!("--include {}", replacements.display()),
        )
        .build();

    cxx_build::bridge("src/melon/sys.rs")
        .include("melonDS/src")
        .include("src/melon/cpp")
        .include("melonDS/src/frontend/glad")
        .file("src/melon/cpp/Platform.cpp")
        .file("src/melon/cpp/Shims.cpp")
        .file("src/melon/cpp/Util.cpp")
        .file("melonDS/src/frontend/glad/glad.c")
        // .define("GDBSTUB_ENABLED", None)
        .flag_if_supported("-std=c++17")
        .compile("melon-bindings"); // arbitrary library name, pick anything

    // link it!
    println!(
        "cargo:rustc-link-search=native={}",
        dst.join("build/src").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        dst.join("build/src/teakra/src").display()
    );
    println!("cargo:rustc-link-lib=static=core");
    println!("cargo:rustc-link-lib=static=teakra");
}
