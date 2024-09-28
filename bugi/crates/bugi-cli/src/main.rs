use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use wasmparser::{ExternalKind, Parser as WasmParser, Payload};

const CALLED_FUNC_PREFIX: &str = "__bugi_v0_called_func_";
const PROVIDE_DESC_NAME: &str = "__bugi_v0_provide_desc";

#[derive(Parser, Debug)]
struct Args {
    /// Path to the Plugin file
    plug_file: PathBuf,

    /// Verbose log
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Check,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let buf = std::fs::read(&args.plug_file).unwrap();

    match args.cmd {
        Commands::Check => {
            check_command(&buf, &args)?;
        }
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Default)]
struct CheckResult {
    exported_funcs: Vec<ExportedFuncData>,
    provided_desc: Option<ExportedFuncData>,
}

#[derive(Debug, Clone, PartialEq)]
enum ExportedFuncData {
    Ok { name: String },
    Err { name: String, msg: &'static str },
}

impl CheckResult {
    fn show(&self) {
        println!("{:?}", self);
    }
}

fn check_command(buf: &[u8], _args: &Args) -> Result<()> {
    let payloads = WasmParser::new(0).parse_all(buf);
    let mut result = CheckResult::default();
    for payload in payloads {
        match payload? {
            Payload::ExportSection(sec) => {
                for exp in sec {
                    let exp = exp?;
                    if exp.name.starts_with(CALLED_FUNC_PREFIX) {
                        if let ExternalKind::Func = exp.kind {
                            let suffix = &exp.name[CALLED_FUNC_PREFIX.len()..];
                            result.exported_funcs.push(ExportedFuncData::Ok {
                                name: suffix.to_string(),
                            });
                        } else {
                            result.exported_funcs.push(ExportedFuncData::Err {
                                name: exp.name.to_string(),
                                msg: "this is not a function",
                            });
                        }
                    } else if exp.name == PROVIDE_DESC_NAME {
                        if let ExternalKind::Func = exp.kind {
                            result.provided_desc = Some(ExportedFuncData::Ok {
                                name: exp.name.to_string(),
                            });
                            //TODO: check function to run
                        } else {
                            result.provided_desc = Some(ExportedFuncData::Err {
                                name: exp.name.to_string(),
                                msg: "this is not a function",
                            });
                        }
                    }
                }
                break;
            }
            _ => continue,
        }
    }

    result.show();

    Ok(())
}
