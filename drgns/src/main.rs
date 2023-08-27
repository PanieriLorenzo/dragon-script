use clap::{Error, Parser};
use error_handler as eh;
use std::{
    io::Write,
    process::{exit, ExitCode},
    sync::{OnceLock, OnceState, RwLock},
};

mod error_handler;

// TODO: overwrite built-in error handling for consistent style
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input: Option<String>,

    #[arg(short, long, action, default_value_t = false, required = false)]
    check: bool,
}

fn main() -> ! {
    let args = Args::parse();

    // once main is done parsing cli arguments, we move execution to the
    // appropriate runners. These runners never return.
    if let Some(path) = args.input.as_deref() {
        // run in batch mode
        run_file(path);
    } else {
        // run in REPL mode
        run_prompt();
    }
}

fn run_file(path: &str) -> ! {
    // define an error handler here for convenience
    let errexit = || -> ! {
        std::process::exit(eh::display_errors());
    };

    let source = match std::fs::read_to_string(path) {
        Ok(src) => src,
        Err(err) => match err.kind() {
            std::io::ErrorKind::NotFound => {
                eh::err_io_not_found(path);
                errexit();
            }
            _ => eh::fatal_unreachable(),
        },
    };
    run(source);
    errexit();
}

fn run_prompt() -> ! {
    loop {
        // TODO: fancy prompt
        print!("> ");
        std::io::stdout().flush().unwrap_or_else(|_| {
            eh::fatal_io_generic("stdout cannot be written to");
        });
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap_or_else(|_| {
            eh::fatal_io_generic("stdout cannot be read");
        });
        run(input);
        eh::display_errors();
    }
}

fn run(source: String) {}
