use vfw_build_utils::{HeaderDir, LinkFile};

fn main() {
    LinkFile::new("terminus_cosim.x")
        .unwrap()
        .add_file("src/link.x")
        .unwrap();
    HeaderDir::new()
        .unwrap()
        .add_dir("include")
        .unwrap()
        .add_dep("vfw_rs")
        .unwrap();
    println!("cargo:rerun-if-changed=build.rs");
}
