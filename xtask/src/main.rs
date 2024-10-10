use std::path::PathBuf;

use anyhow::{Context, Ok, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use xshell::{cmd, Shell};

#[derive(Debug, Parser)]
struct Cli {
    #[clap(subcommand)]
    subcmd: Command,
}

// TODO how to handle mac OS
/// The architecture to compile for.
#[derive(Clone, Debug, ValueEnum)]
enum Arch {
    All,
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
            Arch::All => unreachable!(),
        }
    }
}

impl Arch {
    fn filename(&self) -> &'static str {
        match self {
            Arch::Aarch64 | Arch::X86_64 => "libseda_tally_vm.so",
            Arch::StaticAarch64 | Arch::StaticX86_64 => "libseda_tally_vm.a",
            Arch::All => unreachable!(),
        }
    }
}

/// The commands that can be run.
#[derive(Debug, Subcommand)]
enum Command {
    AptInstall(AptInstall),
    Compile(Compile),
}

/// Installs the necessary packages for the given architecture using apt.
#[derive(Debug, Args)]
struct AptInstall {
    arch: Arch,
}

impl AptInstall {
    fn handle(self, sh: &Shell) -> Result<()> {
        match self.arch {
            Arch::All => {
                cmd!(
                    sh,
                    "sudo apt-get install -y clang llvm gcc-aarch64-linux-gnu qemu-user-static musl-tools"
                )
                .run()?;
            }
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

/// Compiles the libtallyvm for the given architecture.
#[derive(Debug, Args)]
struct Compile {
    #[clap(short, long)]
    debug: bool,
    arch:  Arch,
}
impl Compile {
    fn handle(self, sh: &Shell) -> Result<()> {
        if let Arch::All = self.arch {
            Self::handle_arch(sh, Arch::Aarch64, self.debug)?;
            Self::handle_arch(sh, Arch::X86_64, self.debug)?;
            Self::handle_arch(sh, Arch::StaticAarch64, self.debug)?;
            Self::handle_arch(sh, Arch::StaticX86_64, self.debug)?;
        } else {
            Self::handle_arch(sh, self.arch, self.debug)?;
        }
        Ok(())
    }

    fn handle_arch(sh: &Shell, arch: Arch, debug: bool) -> Result<()> {
        let target = arch.as_ref();

        add_target_if_needed(sh, target)?;
        let rename = match arch {
            Arch::All => unreachable!(),
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

                "libseda_tally_vm.aarch64.so"
            }
            Arch::X86_64 => {
                std::env::set_var("CC", "clang");
                std::env::set_var("CXX", "clang++");
                "libseda_tally_vm.x86_64.so"
            }
            Arch::StaticAarch64 => {
                std::env::set_var("CC", "/opt/aarch64-linux-musl-cross/bin/aarch64-linux-musl-gcc");
                "libseda_tally_vm_muslc.aarch64.a"
            }
            Arch::StaticX86_64 => "libseda_tally_vm_muslc.x86_64.a",
        };

        let path = if debug {
            cmd!(sh, "cargo build --lib --target {target} --locked").run()?;
            "debug"
        } else {
            cmd!(sh, "cargo build --release --lib --target {target}  --locked").run()?;
            "release"
        };

        let target_dir = PathBuf::from("target");
        std::fs::rename(
            target_dir.join(target).join(path).join(arch.filename()),
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
        Command::AptInstall(apt_install) => {
            apt_install.handle(&sh)?;
        }
        Command::Compile(compile) => {
            compile.handle(&sh)?;
        }
    }

    Ok(())
}
