use std::{
    io::Write,
    process::{Command, Output},
};

use clap::{Parser, Subcommand};

#[derive(Parser)]
enum Cli {
    Test {
        #[arg(short, long)]
        force: bool,
    },
    #[command(subcommand)]
    Bugi(BugiCmd),
}

#[derive(Subcommand)]
enum BugiCmd {
    Wasm,
}

const WASM_PLUGS: [&str; 1] = ["wasm-plug"];

fn main() {
    std::env::set_current_dir(format!("{}/../", env!("CARGO_MANIFEST_DIR"))).unwrap();

    let cmd = Cli::parse();
    match cmd {
        Cli::Test { force } => {
            if !exists("./bugi/bugi-tests/wasm-plug.test.wasm") || force {
                bugi_wasm_test_build();
            }

            out(Command::new("cargo")
                .arg("nextest")
                .arg("run")
                .arg("--workspace")
                .arg("--exclude")
                .args(WASM_PLUGS)
                .output()
                .unwrap());
        }
        Cli::Bugi(bugi) => match bugi {
            BugiCmd::Wasm => {
                bugi_wasm_test_build();
            }
        },
    }
}

fn out(out: Output) {
    std::io::stdout().write_all(&out.stdout).unwrap();
    std::io::stderr().write_all(&out.stderr).unwrap();
}

fn exists(path: &str) -> bool {
    std::path::PathBuf::from(path).exists()
}

fn bugi_wasm_test_build() {
    out(Command::new("cargo")
        .arg("build")
        .arg("-r")
        .args(["-p", "wasm-plug"])
        .args(["--target", "wasm32-unknown-unknown"])
        .args(["-Z", "unstable-options"])
        .args(["--artifact-dir", "./.test"])
        .env("RUSTFLAGS", "-Ctarget-feature=+multivalue")
        .output()
        .unwrap());

    std::fs::copy(
        "./.test/wasm_plug.wasm",
        "./bugi/bugi-tests/wasm-plug.test.wasm",
    )
    .unwrap();
}
