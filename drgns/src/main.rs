#![allow(clippy::zero_prefixed_literal)]
// TODO: remove in production
#![allow(dead_code)]
#![warn(clippy::unwrap_used)]
#![feature(trait_alias)]
#![feature(type_alias_impl_trait)]

use clap;
use error_handler as eh;
use lexer::TokenType;
use parser::{BinExpression, Expression, Parser};
use source::SourceArena;
use std::{
    cell::OnceCell,
    io::Write,
    process::exit,
    sync::{Arc, OnceLock, RwLock},
};

use crate::lexer::Lexer;

mod data;
mod error_handler;
mod errors;
mod lexer;
mod lookahead;
mod parser;
mod source;

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
    let mut src = Arc::new(SourceArena::new());
    let lx = Lexer::new(source::Reader::from_arena(&src));
    let mut pr = Parser::new(lx);

    // let e = Expression::BinExpression(BinExpression {
    //     lhs: Box::new(Expression::IntLiteral(42)),
    //     op: parser::BinOperator::Pow,
    //     rhs: Box::new(Expression::BinExpression(BinExpression {
    //         lhs: Box::new(Expression::IntLiteral(22)),
    //         op: parser::BinOperator::Pow,
    //         rhs: Box::new(Expression::IntLiteral(11)),
    //     })),
    // });

    // let e = pr.parse_expression().unwrap();
    // println!("{}", e);

    // exit(0);
    // once main is done parsing cli arguments, we move execution to the
    // appropriate runners. These runners never return.
    if let Some(path) = args.input.as_deref() {
        // run in batch mode
        run_file(src, pr, path);
    } else {
        // run in REPL mode
        run_prompt(src, pr);
    }
}

fn run_file(src: Arc<SourceArena>, mut pr: Parser, path: &str) -> ! {
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
    run(&src, &mut pr, source);
    errexit();
}

fn run_prompt(src: Arc<SourceArena>, mut pr: Parser) -> ! {
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
        run(&src, &mut pr, input);
        eh::display_errors();
    }
}

fn run(src: &Arc<SourceArena>, pr: &mut Parser, input: String) {
    src.intern(input);
    println!("{}", pr.parse_expression().unwrap());
}
