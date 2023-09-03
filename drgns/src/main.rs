use arena::intern;
use clap::{Error, Parser};
use error_handler as eh;
use std::{
    io::Write,
    process::{exit, ExitCode},
    sync::{OnceLock, OnceState, RwLock},
};

use crate::lexer::Lexer;

mod arena;
mod data;
mod error_handler;
mod lexer;

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
    let mut lx = Lexer::new(arena::Reader::new());

    // once main is done parsing cli arguments, we move execution to the
    // appropriate runners. These runners never return.
    if let Some(path) = args.input.as_deref() {
        // run in batch mode
        run_file(&mut lx, path);
    } else {
        // run in REPL mode
        run_prompt(&mut lx);
    }
}

fn run_file(lx: &mut Lexer, path: &str) -> ! {
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
    run(lx, source);
    errexit();
}

fn run_prompt(lx: &mut Lexer) -> ! {
    loop {
        // TODO: fancy prompt
        print!("{}> ", lx.delim_depth() - 1);
        std::io::stdout().flush().unwrap_or_else(|_| {
            eh::fatal_io_generic("stdout cannot be written to");
        });
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap_or_else(|_| {
            eh::fatal_io_generic("stdout cannot be read");
        });
        run(lx, input);
        eh::display_errors();
    }
}

fn run(lx: &mut Lexer, source: String) {
    arena::intern(source);
    while let Some(t) = lx.next() {
        println!("{}", t);
    }
}
