use std::path::PathBuf;

use anyhow::{Context, Ok, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use xshell::{cmd, Shell};

#[derive(Debug, Parser)]
struct Cli {
    #[clap(subcommand)]
    subcmd: Command,
}

#[derive(Clone, Debug, ValueEnum)]
enum Arch {
    Aarch64,
    X86_64,
    StaticAarch64,
    StaticX86_64,
}

impl AsRef<str> for Arch {
    fn as_ref(&self) -> &str {
        match self {
            Arch::Aarch64 => "aarch64-unknown-linux-gnu",
            Arch::X86_64 => "x86_64-unknown-linux-gnu",
            Arch::StaticAarch64 => "aarch64-unknown-linux-musl",
            Arch::StaticX86_64 => "x86_64-unknown-linux-musl",
        }
    }
}

#[derive(Debug, Subcommand)]
enum Command {
    CiInstall(CiInstall),
    Compile(Compile),
}

#[derive(Debug, Args)]
struct CiInstall {
    arch: Arch,
}

impl CiInstall {
    fn handle(self, sh: &Shell) -> Result<()> {
        match self.arch {
            Arch::Aarch64 => {
                cmd!(
                    sh,
                    "sudo apt-get install -y clang llvm gcc-aarch64-linux-gnu qemu-user-static"
                )
                .run()?;
            }
            Arch::X86_64 => {
                cmd!(sh, "sudo apt-get install -y clang llvm").run()?;
            }
            Arch::StaticAarch64 => {
                cmd!(sh, "sudo apt-get install -y musl-tools").run()?;
            }
            Arch::StaticX86_64 => {
                cmd!(sh, "sudo apt-get install -y musl-tools").run()?;
            }
        }

        Ok(())
    }
}

fn add_target_if_needed(sh: &Shell, target: &str) -> Result<()> {
    cmd!(sh, "rustup target add {target}").run()?;
    Ok(())
}

#[derive(Debug, Args)]
struct Compile {
    #[clap(short, long)]
    debug: bool,
    arch:  Arch,
}
impl Compile {
    fn handle(self, sh: &Shell) -> Result<()> {
        let target = self.arch.as_ref();
        let (build_type, path) = if self.debug {
            ("build", "debug")
        } else {
            ("build --release", "release")
        };

        add_target_if_needed(sh, target)?;
        let rename = match self.arch {
            Arch::Aarch64 => {
                std::env::set_var("CC", "clang");
                std::env::set_var("CXX", "clang++");
                std::env::set_var("qemu_aarch64", "qemu-aarch64 -L /usr/aarch64-linux-gnu");
                std::env::set_var("CC_aarch64_unknown_linux_gnu", "clang");
                std::env::set_var("AR_aarch64_unknown_linux_gnu", "llvm-ar");
                std::env::set_var("CFLAGS_aarch64_unknown_linux_gnu", "--sysroot=/usr/aarch64-linux-gnu");
                std::env::set_var("CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER", "aarch64-linux-gnu-gcc");
                std::env::set_var(
                    "CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUNNER",
                    "qemu-aarch64 -L /usr/aarch64-linux-gnu",
                );

                "libseda_tally_vm.aarch64.a"
            }
            Arch::X86_64 => {
                std::env::set_var("CC", "clang");
                std::env::set_var("CXX", "clang++");
                "libseda_tally_vm.x86_64.a"
            }
            Arch::StaticAarch64 => {
                std::env::set_var("CC", "/opt/aarch64-linux-musl-cross/bin/aarch64-linux-musl-gcc");
                "libseda_tally_vm.aarch64.static.a"
            }
            Arch::StaticX86_64 => "libseda_tally_vm.x86_64.static.a",
        };
        cmd!(sh, "cargo {build_type} --lib --target {target} --locked").run()?;

        let target_dir = PathBuf::from("target");
        std::fs::rename(
            target_dir.join(target).join(path).join("libseda_tally_vm.a"),
            target_dir.join(rename),
        )?;
        Ok(())
    }
}

fn main() {
    if let Err(e) = try_main() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn try_main() -> Result<()> {
    let sh = Shell::new()?;

    // Ensure our working directory is the toplevel
    {
        let path = cmd!(&sh, "git rev-parse --show-toplevel")
            .read()
            .context("Faild to invoke git rev-parse")?;
        std::env::set_current_dir(path.trim()).context("Changing to toplevel")?;
    }

    let args = Cli::parse();

    match args.subcmd {
        Command::CiInstall(ci_install) => {
            ci_install.handle(&sh)?;
        }
        Command::Compile(compile) => {
            compile.handle(&sh)?;
        }
    }

    Ok(())
}
