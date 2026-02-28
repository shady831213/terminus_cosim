use std::env;
use vfw_build_utils::*;
fn main() {
    let mut c_build = cc::Build::new();
    let out_dir = env::var("OUT_DIR").unwrap();
    let toolchain_prefix = env::var("RISCV_TOOLCHAIN_PREFIX").unwrap_or("".to_string());

    if let Some(build) = build_c_files("c", &mut c_build).unwrap() {
        build
            .compiler(format!("{}gcc", &toolchain_prefix))
            .archiver(format!("{}ar", &toolchain_prefix))
            .out_dir(out_dir)
            // .flag("-mabi=lp64d")
            // .flag("-mcmodel=medany")
            .flag("-Wno-main")
            .flag("-Wno-strict-aliasing")
            .flag("-Wno-builtin-declaration-mismatch")
            .flag("-Wno-varargs")
            .compile("c_stubs");
    }
    println!("cargo:rerun-if-changed=build.rs");
}
