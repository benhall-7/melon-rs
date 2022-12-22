use cmake::Config;

fn main() {
    // build melonDS
    let dst = Config::new("melonDS")
        .define("BUILD_QT_SDL", "OFF")
        .build_target("all")
        .build();

    // bindings are compiled based on header files and dynamically added
    // based on which functions we use, and those others they depends on
    let path = std::path::PathBuf::from("melonDS/src"); // include path
    cxx_build::bridge("src/melon/sys.rs")
        .include(path)
        .flag_if_supported("-std=c++17")
        .compile("melon-bindings"); // arbitrary library name, pick anything
    println!("cargo:rerun-if-changed=src/melon/sys.rs");

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
