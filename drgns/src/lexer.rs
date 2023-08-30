use std::{
    cell::OnceCell,
    collections::HashMap,
    sync::{OnceLock, RwLock},
};

use append_only_vec::AppendOnlyVec;
use clap::error;

use crate::{
    arena::{Reader, Span},
    data,
    data::PrimitiveValue,
    error_handler,
};

use regex::Regex;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // unambiguously single-character tokens
    SingleQuote,
    DoubleQuote,
    Caret,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Comma,
    Semicolon,
    Pipe,

    // one or more chars
    Plus,
    PlusPlus,
    PlusEquals,
    PlusPlusEquals,
    Mul,
    MulMul,
    MulEquals,
    MulMulEquals,
    Minus,
    MinusEquals,
    Arrow,
    Div,
    DivEquals,
    Mod,
    ModEquals,
    Less,
    LessEquals,
    DoubleLess,
    Greater,
    GreaterEquals,
    DoubleGreater,
    Colon,
    ColonColon,
    ColonEquals,
    Bang,
    BangEquals,
    Question,
    DoubleQuestion,
    DoubleQuestionEqual,
    Equal,
    EqualEqual,
    Dot,
    Ellipses,
    DollarBrace,

    // literals
    Identifier,
    Symbol,
    String,
    RawString,
    IntLit,
    FloatLit,

    // Keywords
    And,
    Asr,
    Bool,
    Continue,
    Copy,
    False,
    Float,
    For,
    Function,
    Int,
    In,
    Land,
    List,
    Lnot,
    Lor,
    Lsl,
    Lsr,
    Lxor,
    Move,
    Mut,
    None_,
    Not,
    Obj,
    Or,
    Return,
    Str,
    Sym,
    True,
    Underscore,
    Use,
    Xor,

    // whitespace, comments and already handled tokens
    Ignore,

    // unrecognized tokens
    Unknown,

    // a "fake" character emitted at the end of the stream
    // TODO: this is unused? just use Option<Token>::None
    EOF,
}

static KEYWORDS: OnceLock<RwLock<HashMap<&'static str, TokenType>>> = OnceLock::new();

fn init_keywords() {
    KEYWORDS.get_or_init(|| {
        RwLock::new(
            [
                ("and", TokenType::And),
                ("asr", TokenType::Asr),
                ("bool", TokenType::Bool),
                ("continue", TokenType::Continue),
                ("copy", TokenType::Copy),
                ("false", TokenType::False),
                ("float", TokenType::Float),
                ("for", TokenType::For),
                ("function", TokenType::Function),
                ("int", TokenType::Int),
                ("in", TokenType::In),
                ("land", TokenType::Land),
                ("list", TokenType::List),
                ("lnot", TokenType::Lnot),
                ("lor", TokenType::Lor),
                ("lsl", TokenType::Lsl),
                ("lsr", TokenType::Lsr),
                ("lxor", TokenType::Lxor),
                ("move", TokenType::Move),
                ("mut", TokenType::Mut),
                ("none", TokenType::None_),
                ("not", TokenType::Not),
                ("obj", TokenType::Obj),
                ("or", TokenType::Or),
                ("return", TokenType::Return),
                ("str", TokenType::Str),
                ("sym", TokenType::Sym),
                ("true", TokenType::True),
                ("_", TokenType::Underscore),
                ("use", TokenType::Use),
                ("xor", TokenType::Xor),
            ]
            .iter()
            .cloned()
            .collect(),
        )
    });
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: Span,
    literal: Option<data::PrimitiveValue>,
}

impl ToString for Token {
    fn to_string(&self) -> String {
        return format!(
            "({}, {}, {})",
            self.token_type,
            self.lexeme,
            match &self.literal {
                None => "".to_owned(),
                Some(x) => x.to_string(),
            }
        );
    }
}

// struct Lexer {
//     source: String,
//     tokens: Vec<Token>,
//     start: usize,
//     current: usize,
//     line: usize,
// }

pub struct Lexer {
    reader: Reader,
}

impl Lexer {
    pub fn new(reader: Reader) -> Self {
        Self { reader }
    }

    /// parses all tokens that start with +
    fn ambiguous_plus(&mut self) -> (TokenType, Option<PrimitiveValue>) {
        use crate::lexer::TokenType::*;
        match (self.reader.peek(), self.reader.peek2()) {
            (Some('+'), Some('=')) => {
                self.reader.next();
                self.reader.next();
                (PlusPlusEquals, None)
            }
            (Some('+'), _) => {
                self.reader.next();
                (PlusPlus, None)
            }
            (Some('='), _) => {
                self.reader.next();
                (PlusEquals, None)
            }
            _ => (Plus, None),
        }
    }

    /// parses all tokens that start with *
    fn ambiguous_mul(&mut self) -> (TokenType, Option<PrimitiveValue>) {
        use crate::lexer::TokenType::*;
        match (self.reader.peek(), self.reader.peek2()) {
            (Some('*'), Some('=')) => {
                self.reader.next();
                self.reader.next();
                (MulMulEquals, None)
            }
            (Some('*'), _) => {
                self.reader.next();
                (MulMul, None)
            }
            (Some('='), _) => {
                self.reader.next();
                (MulEquals, None)
            }
            _ => (Mul, None),
        }
    }

    /// parses all tokens that start with -
    fn ambiguous_minus(&mut self) -> (TokenType, Option<PrimitiveValue>) {
        use crate::lexer::TokenType::*;
        match (self.reader.peek(), self.reader.peek2()) {
            (Some('='), _) => {
                self.reader.next();
                (MinusEquals, None)
            }
            (Some('>'), _) => {
                self.reader.next();
                (Arrow, None)
            }
            _ => (Minus, None),
        }
    }

    /// parses all tokens that start with a /
    fn ambiguous_slash(&mut self) -> (TokenType, Option<PrimitiveValue>) {
        use crate::lexer::TokenType::*;
        match (self.reader.peek(), self.reader.peek2()) {
            (Some('='), _) => {
                self.reader.next();
                (DivEquals, None)
            }
            // comment
            (Some('/'), _) => {
                while self.reader.peek() != Some('\n') && self.reader.peek().is_some() {
                    self.reader.next();
                }
                (Ignore, None)
            }
            _ => (Div, None),
        }
    }

    /// parses all tokens that start with a %
    fn ambiguous_mod(&mut self) -> (TokenType, Option<PrimitiveValue>) {
        use crate::lexer::TokenType::*;
        match (self.reader.peek(), self.reader.peek2()) {
            (Some('='), _) => {
                self.reader.next();
                (ModEquals, None)
            }
            _ => (Mod, None),
        }
    }

    /// parses all tokens that start with <
    fn ambiguous_less(&mut self) -> (TokenType, Option<PrimitiveValue>) {
        use crate::lexer::TokenType::*;
        match (self.reader.peek(), self.reader.peek2()) {
            (Some('<'), _) => {
                self.reader.next();
                (DoubleLess, None)
            }
            (Some('='), _) => {
                self.reader.next();
                (LessEquals, None)
            }
            _ => (Less, None),
        }
    }

    /// parses all tokens that start with >
    fn ambiguous_greater(&mut self) -> (TokenType, Option<PrimitiveValue>) {
        use crate::lexer::TokenType::*;
        match (self.reader.peek(), self.reader.peek2()) {
            (Some('>'), _) => {
                self.reader.next();
                (DoubleGreater, None)
            }
            (Some('='), _) => {
                self.reader.next();
                (GreaterEquals, None)
            }
            _ => (Greater, None),
        }
    }

    /// parses all tokens that start with :
    fn ambiguous_colon(&mut self) -> (TokenType, Option<PrimitiveValue>) {
        use crate::lexer::TokenType::*;
        match (self.reader.peek(), self.reader.peek2()) {
            (Some(':'), _) => {
                self.reader.next();
                (ColonColon, None)
            }
            (Some('='), _) => {
                self.reader.next();
                (ColonEquals, None)
            }
            _ => (Colon, None),
        }
    }

    /// parses all tokens that start with a !
    fn ambiguous_bang(&mut self) -> (TokenType, Option<PrimitiveValue>) {
        use crate::lexer::TokenType::*;
        match (self.reader.peek(), self.reader.peek2()) {
            (Some('='), _) => {
                self.reader.next();
                (BangEquals, None)
            }
            _ => (Bang, None),
        }
    }

    /// parses all tokens that start with ?
    fn ambiguous_question(&mut self) -> (TokenType, Option<PrimitiveValue>) {
        use crate::lexer::TokenType::*;
        match (self.reader.peek(), self.reader.peek2()) {
            (Some('?'), Some('=')) => {
                self.reader.next();
                self.reader.next();
                (DoubleQuestionEqual, None)
            }
            (Some('?'), _) => {
                self.reader.next();
                (DoubleQuestion, None)
            }
            _ => (Question, None),
        }
    }

    /// parses all tokens that start with a =
    fn ambiguous_equal(&mut self) -> (TokenType, Option<PrimitiveValue>) {
        use crate::lexer::TokenType::*;
        match (self.reader.peek(), self.reader.peek2()) {
            (Some('='), _) => {
                self.reader.next();
                (EqualEqual, None)
            }
            _ => (Equal, None),
        }
    }

    /// parses all tokens that start with a .
    fn ambiguous_dot(&mut self) -> (TokenType, Option<PrimitiveValue>) {
        use crate::lexer::TokenType::*;
        match (self.reader.peek(), self.reader.peek2()) {
            (Some('.'), Some('.')) => {
                self.reader.next();
                self.reader.next();
                (Ellipses, None)
            }
            _ => (Dot, None),
        }
    }

    fn lex_number_literal(&mut self) -> (TokenType, Option<PrimitiveValue>) {
        let mut is_float = false;

        // helper for matching digit or digit separator, e.g. 123_456_789
        let is_digit_or_sep = |c: char| c.is_ascii_digit() || c == '_';

        // match integer part
        while self.reader.peek().is_some_and(is_digit_or_sep) {
            self.reader.next();
        }

        // look for a fractional part
        if self.reader.peek() == Some('.') && self.reader.peek2().is_some_and(is_digit_or_sep) {
            is_float = true;
            // consume the "."
            self.reader.next();

            while self.reader.peek().is_some_and(is_digit_or_sep) {
                self.reader.next();
            }

            // TODO: look for scientific notation
        }

        if is_float {
            let val = self
                .reader
                .current
                .to_string()
                .replace("_", "")
                .parse()
                .unwrap_or_else(|_| error_handler::fatal_unreachable());
            (TokenType::FloatLit, Some(PrimitiveValue::Float(val)))
        } else {
            let val = self
                .reader
                .current
                .to_string()
                .replace("_", "")
                .parse()
                .unwrap_or_else(|_| {
                    error_handler::err_int_too_big();
                    0
                });
            (TokenType::IntLit, Some(PrimitiveValue::Int(val)))
        }
    }
}

impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        use TokenType::*;
        let c = self.reader.next()?;
        let (token_type, literal) = match c {
            // match unambiguously single-character tokens
            '\'' => (SingleQuote, None),
            '"' => (DoubleQuote, None),
            '^' => (Caret, None),
            '(' => (LeftParen, None),
            ')' => (RightParen, None),
            '[' => (LeftBracket, None),
            ']' => (RightBracket, None),
            '{' => (LeftBrace, None),
            '}' => (RightBrace, None),
            ',' => (Comma, None),
            ';' => (Semicolon, None),
            '|' => (Pipe, None),

            // match one or more character tokens
            '+' => self.ambiguous_plus(),
            '*' => self.ambiguous_mul(),
            '-' => self.ambiguous_minus(),
            '%' => self.ambiguous_mod(),
            '<' => self.ambiguous_less(),
            '>' => self.ambiguous_greater(),
            ':' => self.ambiguous_colon(),
            '!' => self.ambiguous_bang(),
            '?' => self.ambiguous_question(),
            '=' => self.ambiguous_equal(),
            '.' => self.ambiguous_dot(),

            // includes comment vvv
            '/' => self.ambiguous_slash(),

            // ignore whitespace
            ' ' | '\n' | '\r' | '\t' => (Ignore, None),

            // TODO: string literals, requires modal lexing

            // numbers
            c if c.is_ascii_digit() => self.lex_number_literal(),

            _ => error_handler::fatal_unreachable(),
        };
        Some(Token {
            token_type,
            lexeme: self.reader.advance_tail(),
            literal: literal,
        })
    }
}
