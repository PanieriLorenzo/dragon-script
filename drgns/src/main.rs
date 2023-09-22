#![allow(clippy::zero_prefixed_literal)]
// TODO: remove in production
#![allow(dead_code)]
#![feature(trait_alias)]
#![feature(type_alias_impl_trait)]

use clap;
use error_handler as eh;
use lexer::TokenType;
use parser::{BinExpression, Expression, Parser};
use std::{io::Write, process::exit};

use crate::lexer::Lexer;

mod arena;
mod data;
mod error_handler;
mod lexer;
mod lookahead;
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
    let mut pr = Parser::new(Lexer::new(arena::Reader::new()));

    // let e = Expression::BinExpression(BinExpression {
    //     lhs: Box::new(Expression::IntLiteral(42)),
    //     op: parser::BinOperator::Pow,
    //     rhs: Box::new(Expression::BinExpression(BinExpression {
    //         lhs: Box::new(Expression::IntLiteral(22)),
    //         op: parser::BinOperator::Pow,
    //         rhs: Box::new(Expression::IntLiteral(11)),
    //     })),
    // });

    arena::intern("-2 ** --4".into());
    let e = pr.parse_expression().unwrap();
    println!("{}", e);

    exit(0);
    // once main is done parsing cli arguments, we move execution to the
    // appropriate runners. These runners never return.
    if let Some(path) = args.input.as_deref() {
        // run in batch mode
        run_file(pr, path);
    } else {
        // run in REPL mode
        run_prompt(pr);
    }
}

fn run_file(mut pr: Parser, path: &str) -> ! {
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
    run(&mut pr, source);
    errexit();
}

fn run_prompt(mut pr: Parser) -> ! {
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
        run(&mut pr, input);
        eh::display_errors();
    }
}

fn run(pr: &mut Parser, source: String) {
    arena::intern(source);
    // println!("{:?}", pr.match_un_expression());
}
