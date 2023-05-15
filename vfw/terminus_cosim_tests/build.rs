use std::env;
use vfw_build_utils::*;
fn main() {
    tests_build_with(
        &env::var("RISCV_TOOLCHAIN_PREFIX").unwrap_or("".to_string()),
        |builder| {
            let (_, incdir) = c_src_dir(&dep_header("terminus_cosim").unwrap()).unwrap();
            if env::var("CARGO_CFG_TARGET_ARCH").unwrap() == "riscv64" {
                builder.flag("-mabi=lp64d")
            } else {
                builder.flag("-mabi=ilp32d")
            }
            .flag("-lto")
            .includes(&incdir)
        },
    );
    println!("cargo:rerun-if-changed=build.rs");
}
