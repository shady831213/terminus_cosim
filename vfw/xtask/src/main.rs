use anyhow::Result;
use vfw_xtask::*;
struct Vfw;
impl Target for Vfw {
    fn name(&self) -> &str {
        "vfw"
    }
    fn prj_dir(&self) -> std::path::PathBuf {
        prj_dir!()
    }
    fn args(&self) -> clap::Command {
        self.common_args().mut_subcommand("build", |build| {
            build.arg(
                clap::Arg::new("rv64")
                    .long("64")
                    .action(clap::ArgAction::SetTrue),
            )
        })
    }
    fn build_handler(&self, test_name: &str, m: &clap::ArgMatches) -> Result<()> {
        let rv64 = *m.get_one::<bool>("rv64").unwrap();
        let mut cargo = std::process::Command::new("cargo");
        cargo.arg("+nightly");
        println!(
            "compile test {} {}...",
            if rv64 { "rv64" } else { "rv32" },
            test_name
        );
        self.run_build(
            self.common_build_cmd(cargo, test_name)
                .args(["-p", "tests"])
                .args([
                    "--target",
                    if rv64 {
                        "riscv64gc-unknown-none-elf"
                    } else {
                        "riscv32imac-unknown-none-elf"
                    },
                ])
                .env("RISCV_TOOLCHAIN_PREFIX", "riscv64-unknown-elf-")
                .arg(&format!(
                    "--features={}",
                    if rv64 {
                        "terminus_cosim/ptr64"
                    } else {
                        "terminus_cosim/ptr32"
                    },
                )),
        )?;
        self.run_dump(test_name, "riscv64-unknown-elf-")?;
        println!(
            "compile test {} {} done!",
            if rv64 { "rv64" } else { "rv32" },
            test_name
        );
        Ok(())
    }
}

fn main() -> Result<()> {
    Xtask::new().add_target(Vfw).run()
}
