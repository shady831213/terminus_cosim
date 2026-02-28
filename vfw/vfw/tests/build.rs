use std::env;
use vfw_build_utils::*;
use which::which;
fn main() {
    let toolchain_prefix = env::var("RISCV_TOOLCHAIN_PREFIX").unwrap_or("".to_string());

    let (_, incdir) = c_src_dir(&dep_header("terminus_cosim").unwrap()).unwrap();
    if env::var("CARGO_FEATURE_C_COV").is_ok() {
        //build c
        let mut c_build = cc::Build::new();
        let out_dir = env::var("OUT_DIR").unwrap();
        if let Some(build) = build_c_files("common_c", &mut c_build).unwrap() {
            if env::var("CARGO_CFG_TARGET_ARCH").unwrap() == "riscv64" {
                build.flag("-mabi=lp64d").flag("-mcmodel=medany")
            } else {
                build
            }
            .compiler(format!("{}gcc", &toolchain_prefix))
            .archiver(format!("{}ar", &toolchain_prefix))
            .includes(&incdir)
            .out_dir(out_dir)
            .flag("-Wno-main")
            .flag("-Wno-strict-aliasing")
            .flag("-Wno-builtin-declaration-mismatch")
            .flag("-Wno-varargs")
            .compile("common_c");
        }
    }
    tests_build_with(&toolchain_prefix, |builder| {
        let builder = if env::var("CARGO_CFG_TARGET_ARCH").unwrap() == "riscv64" {
            builder.flag("-mabi=lp64d").flag("-mcmodel=medany")
        } else {
            builder
        };
        if env::var("CARGO_FEATURE_C_COV").is_ok() {
            builder
                .flag("-fprofile-arcs")
                .flag("-ftest-coverage")
                .flag("-fprofile-info-section")
        } else {
            builder
        }
        .flag("-lto")
        .flag("-Wno-varargs")
        .includes(&incdir)
    });
    if env::var("CARGO_FEATURE_C_COV").is_ok() {
        //expect multi target build toolchain
        let lib_path = which(format!("{}gcc", &toolchain_prefix))
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("lib")
            .join("gcc")
            .join("riscv64-unknown-elf")
            .join("12.1.0");
        let lib_path = if env::var("CARGO_CFG_TARGET_ARCH").unwrap() == "riscv64" {
            lib_path.join("libgcov.a")
        } else {
            lib_path.join("rv32imac").join("ilp32").join("libgcov.a")
        };
        println!("cargo:rustc-link-arg={}", lib_path.display());
    }
    println!("cargo:rerun-if-changed=build.rs");
}
