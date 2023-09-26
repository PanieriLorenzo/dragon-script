#![allow(clippy::zero_prefixed_literal)]
// TODO: remove in production
#![allow(dead_code)]
#![warn(clippy::unwrap_used)]
#![feature(trait_alias)]
#![feature(type_alias_impl_trait)]

use ariadne::{sources, Report, ReportKind};
use clap;
use error_handler as eh;
use lexer::TokenType;
use log::debug;
use parser::{BinExpression, Expression, Parser};
use source::{Reader, SourceArena, SourceView};
use std::{cell::OnceCell, io::Write, process::exit, rc::Rc, sync::RwLock};

use crate::lexer::Lexer;

use miette::Result;

mod data;
mod error_handler;
use error_handler::ErrorHandler;
mod eval;
mod lexer;
mod lookahead;
mod parser;
mod source;
mod values;

// TODO: overwrite built-in error handling for consistent style
#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input: Option<String>,

    #[arg(short, long, action, default_value_t = false, required = false)]
    check: bool,
}

struct Interpreter {
    args: Args,
    src: Rc<SourceArena>,
    pr: Parser,
    eh: Rc<ErrorHandler>,
}

impl Interpreter {
    fn start(&mut self) -> ! {
        if let Some(path) = self.args.input.clone().as_deref() {
            debug!("running in batch mode");
            self.run_file(path);
        } else {
            debug!("running in REPL mode");
            self.run_prompt();
        }
    }

    fn run_file(&mut self, path: &str) -> ! {
        // define an error handler here for convenience
        let errexit = || -> ! {
            std::process::exit(eh::display_errors());
        };

        let source = match std::fs::read_to_string(path) {
            Ok(src) => src,
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => {
                    fatal!(format!("path '{}' does not exist", path));
                }
                _ => assert_unreachable!(),
            },
        };
        self.run(source);
        errexit();
    }

    fn run_prompt(&mut self) -> ! {
        loop {
            // TODO: fancy prompt
            //print!("{}> ", lx.delim_depth() - 1);
            print!("> ");
            std::io::stdout().flush().unwrap_or_else(|_| {
                fatal!("stdout cannot be written to");
            });
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap_or_else(|_| {
                fatal!("stdout cannot be read");
            });
            self.run(input);
            self.eh.report_all();
        }
    }

    fn run(&mut self, input: String) {
        self.src.intern(input);
        let mut eval = eval::ExpressionEval::new();
        self.pr.parse_expression().unwrap().walk(&mut eval);
        println!("{:?}", eval);
    }
}

fn main() -> ! {
    std::env::set_var("RUST_LOG", "trace");
    env_logger::builder().format_timestamp(None).init();
    let src = Rc::new(SourceArena::new());
    let eh = Rc::new(ErrorHandler::new(&src));
    let mut i = Interpreter {
        args: <Args as clap::Parser>::parse(),
        src: src.clone(),
        pr: Parser::new(Lexer::new(source::Reader::from_arena(&src), &eh), &eh),
        eh: eh.clone(),
    };

    i.start();
}
