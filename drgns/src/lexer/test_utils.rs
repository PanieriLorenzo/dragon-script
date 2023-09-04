use std::{
    collections::HashMap,
    sync::{OnceLock, RwLock},
};

use super::{OnceMap, TokenType};

static STR_2_TOKENS: OnceMap<&'static str, TokenType> = OnceLock::new();

fn init_str_2_tokens() -> &'static RwLock<HashMap<&'static str, TokenType>> {
    use TokenType as TT;
    STR_2_TOKENS.get_or_init(|| {
        RwLock::new(
            [
                ("(", TT::LeftParen),
                (")", TT::RightParen),
                ("{", TT::LeftBrace),
                ("}", TT::RightBrace),
                (",", TT::Comma),
                (";", TT::Semicolon),
                ("+", TT::Plus),
                ("*", TT::Mul),
                ("%", TT::Mod),
                ("/", TT::Div),
                ("-", TT::Minus),
                ("->", TT::Arrow),
                ("<", TT::Less),
                ("<=", TT::LessEquals),
                (">", TT::Greater),
                (">=", TT::GreaterEquals),
                ("=", TT::Equal),
                ("==", TT::EqualEqual),
                (":=", TT::ColonEquals),
                ("!=", TT::BangEquals),
                //("", TT::Identifier),
                //("", TT::IntLit),
                ("and", TT::And),
                ("break", TT::Break),
                ("false", TT::False),
                ("for", TT::For),
                ("function", TT::Function),
                ("not", TT::Not),
                ("or", TT::Or),
                ("return", TT::Return),
                ("true", TT::True),
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
        TT::LeftParen => "(",
        TT::RightParen => ")",
        TT::LeftBrace => "{",
        TT::RightBrace => "}",
        TT::Comma => ",",
        TT::Semicolon => ";",
        TT::Plus => "+",
        TT::Mul => "*",
        TT::Mod => "%",
        TT::Div => "/",
        TT::Minus => "-",
        TT::Arrow => "->",
        TT::Less => "<",
        TT::LessEquals => "<=",
        TT::Greater => ">",
        TT::GreaterEquals => ">=",
        TT::Equal => "=",
        TT::EqualEqual => "==",
        TT::ColonEquals => ":=",
        TT::BangEquals => "!=",
        TT::Identifier => "andy",
        TT::IntLit => "42",
        TT::And => "and",
        TT::Break => "break",
        TT::False => "false",
        TT::For => "for",
        TT::Function => "function",
        TT::Not => "not",
        TT::Or => "or",
        TT::Return => "return",
        TT::True => "true",
        TT::Ignore => " ",
        TT::Unknown => "?",
    }
}
