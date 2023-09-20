#![allow(clippy::zero_prefixed_literal)]
// TODO: remove in production
#![allow(dead_code)]

use clap;
use error_handler as eh;
use lexer::TokenType;
use parser::Parser;
use std::io::Write;

use crate::lexer::Lexer;

mod arena;
mod data;
mod error_handler;
mod lexer;
mod parser;

// TODO: overwrite built-in error handling for consistent style
#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input: Option<String>,

    #[arg(short, long, action, default_value_t = false, required = false)]
    check: bool,
}

fn main() -> ! {
    let args = <Args as clap::Parser>::parse();
    let mut lx = Lexer::new(arena::Reader::new());
    let mut ps = Parser::new(lx);

    // once main is done parsing cli arguments, we move execution to the
    // appropriate runners. These runners never return.
    if let Some(path) = args.input.as_deref() {
        // run in batch mode
        run_file(&mut ps, path);
    } else {
        // run in REPL mode
        run_prompt(&mut ps);
    }
}

fn run_file(ps: &mut Parser, path: &str) -> ! {
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
    run(ps, source);
    errexit();
}

fn run_prompt(ps: &mut Parser) -> ! {
    loop {
        // TODO: fancy prompt
        //print!("{}> ", lx.delim_depth() - 1);
        print!("> ");
        std::io::stdout().flush().unwrap_or_else(|_| {
            eh::fatal_io_generic("stdout cannot be written to");
        });
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap_or_else(|_| {
            eh::fatal_io_generic("stdout cannot be read");
        });
        run(ps, input);
        eh::display_errors();
    }
}

fn run(ps: &mut Parser, source: String) {
    arena::intern(source);
    for t in ps {
        println!("{}", t);
    }
}
