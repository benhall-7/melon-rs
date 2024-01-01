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
        .build_target("all")
        .define("INTERCEPT", "ON")
        .define(
            "CMAKE_CXX_FLAGS",
            format!("--include {}", replacements.display()),
        )
        .build();

    let bridge_files = [
        "glue",
        "nds_cart",
        "nds",
        "platform",
        "replacements",
        "shims",
    ];
    cxx_build::bridges(
        bridge_files
            .iter()
            .map(|file| format!("src/melon/sys/{file}.rs")),
    )
    .include("melonDS/src")
    .include("melonDS/src/frontend/glad")
    .include("src/melon/cpp")
    .file("src/melon/cpp/Platform.cpp")
    .file("melonDS/src/frontend/glad/glad.c")
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
