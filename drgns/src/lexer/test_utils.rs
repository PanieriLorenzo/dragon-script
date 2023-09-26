use std::{
    collections::HashMap,
    str::Chars,
    sync::{OnceLock, RwLock},
};

use crate::source::{Reader, SourceArena};

use super::{Lexer, OnceMap, TokenType};

use itertools::{iproduct, Product};

static STR_2_TOKENS: OnceMap<&'static str, TokenType> = OnceLock::new();

fn init_str_2_tokens() -> &'static RwLock<HashMap<&'static str, TokenType>> {
    use TokenType as TT;
    STR_2_TOKENS.get_or_init(|| {
        RwLock::new(
            [
                ("(", TT::LeftParen),
                (")", TT::RightParen),
                (",", TT::Comma),
                ("+", TT::Plus),
                ("*", TT::Star),
                ("**", TT::Pow),
                ("%", TT::Percent),
                ("/", TT::Slash),
                ("-", TT::Minus),
                (":=", TT::ColonEquals),
                //("", TT::Identifier),
                //("", TT::IntLit),
                ("exit", TT::Exit),
                (" ", TT::Ignore),
                ("\t", TT::Ignore),
                ("\r", TT::Ignore),
                ("\n", TT::Ignore),
            ]
            .iter()
            .cloned()
            .collect(),
        )
    })
}

pub fn str_2_tokens(s: &str) -> TokenType {
    *init_str_2_tokens()
        .read()
        .expect("poisoned lock")
        .get(s)
        .unwrap_or_else(|| &TokenType::Unknown)
}

pub fn tokens_2_str(tt: TokenType) -> &'static str {
    use TokenType as TT;
    match tt {
        TT::Semicolon => ";",
        TT::LeftParen => "(",
        TT::RightParen => ")",
        TT::Comma => ",",
        TT::Plus => "+",
        TT::Star => "*",
        TT::Percent => "%",
        TT::Slash => "/",
        TT::Minus => "-",
        TT::ColonEquals => ":=",
        TT::Identifier => "andy",
        TT::IntLit => "42",
        TT::Ignore => " ",
        TT::Unknown => "?",
        TT::Pow => "**",
        TT::Exit => "exit",
    }
}

pub fn lel(s: &str) -> u32 {
    match s {
        "lel" => 1,
        _ => 0,
    }
}

#[macro_export]
/// produces iterator over all sequences of 2 printable ascii characters
macro_rules! two_char_strings {
    () => {{
        let all_ascii = "\n\r\t !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~".to_string();
        iproduct!(all_ascii.clone().chars(), all_ascii.clone().chars()).map(|(s, t)| format!("{}{}", s, t))
    }};
}
