use cmake::Config;

fn main() {
    // build melonDS
    let dst = Config::new("melonDS")
        .define("BUILD_QT_SDL", "OFF")
        .build_target("core")
        .build();

    // bindings are compiled based on header files only, we linked to actual code later
    let path = std::path::PathBuf::from("melonDS/src"); // include path
    let mut b = autocxx_build::Builder::new("src/melon/sys.rs", [&path])
        .build()
        .expect("couldn't build sys :<");

    b.flag_if_supported("-std=c++17")
        .compile("melon-bindings"); // arbitrary library name, pick anything
    println!("cargo:rerun-if-changed=src/melon/sys.rs");

    // link it!
    println!(
        "cargo:rustc-link-search=native={}",
        dst.join("build/src").display()
    );
    println!("cargo:rustc-link-lib=static=core");
}
