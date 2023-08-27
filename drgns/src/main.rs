use clap::{Error, Parser};
use error_handler::{display_errors, err_io_not_found, fatal_unreachable};
use std::{
    process::{exit, ExitCode},
    sync::{OnceLock, OnceState, RwLock},
};

mod error_handler;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input: Option<String>,

    #[arg(short, long, action, default_value_t = false, required = false)]
    check: bool,
}

fn main() {
    let args = Args::parse();
    if let Some(path) = args.input.as_deref() {
        // run in batch mode
        let _ = run_file(path);
    } else {
        // run in REPL mode
        run_prompt();
    }
    std::process::exit(display_errors());
}

fn run_file(path: &str) {
    let source = match std::fs::read_to_string(path) {
        Ok(src) => src,
        Err(err) => match err.kind() {
            std::io::ErrorKind::NotFound => {
                err_io_not_found(path);
                return;
            }
            _ => fatal_unreachable(),
        },
    };
}

fn run_prompt() {}
