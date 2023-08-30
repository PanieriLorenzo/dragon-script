use std::{
    collections::HashMap,
    sync::{OnceLock, RwLock},
};

use append_only_vec::AppendOnlyVec;

use crate::{
    arena::{Reader, Span},
    data, error_handler,
};

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
    Integer,
    Float,

    // Keywords
    And,
    Asr,
    Bool,
    Continue,
    Copy,
    False,
    Float_,
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
                ("float", TokenType::Float_),
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
            "{} {} {}",
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
    fn ambiguous_plus(&mut self) -> TokenType {
        use crate::lexer::TokenType::*;
        match (self.reader.peek(), self.reader.peek2()) {
            (Some('+'), Some('=')) => {
                self.reader.next();
                self.reader.next();
                PlusPlusEquals
            }
            (Some('+'), _) => {
                self.reader.next();
                PlusPlus
            }
            (Some('='), _) => {
                self.reader.next();
                PlusEquals
            }
            _ => Plus,
        }
    }

    /// parses all tokens that start with *
    fn ambiguous_mul(&mut self) -> TokenType {
        use crate::lexer::TokenType::*;
        match (self.reader.peek(), self.reader.peek2()) {
            (Some('*'), Some('=')) => {
                self.reader.next();
                self.reader.next();
                MulMulEquals
            }
            (Some('*'), _) => {
                self.reader.next();
                MulMul
            }
            (Some('='), _) => {
                self.reader.next();
                MulEquals
            }
            _ => Mul,
        }
    }

    /// parses all tokens that start with -
    fn ambiguous_minus(&mut self) -> TokenType {
        use crate::lexer::TokenType::*;
        match (self.reader.peek(), self.reader.peek2()) {
            (Some('='), _) => {
                self.reader.next();
                MinusEquals
            }
            (Some('>'), _) => {
                self.reader.next();
                Arrow
            }
            _ => Minus,
        }
    }

    /// parses all tokens that start with a /
    fn ambiguous_slash(&mut self) -> TokenType {
        use crate::lexer::TokenType::*;
        match (self.reader.peek(), self.reader.peek2()) {
            (Some('='), _) => {
                self.reader.next();
                DivEquals
            }
            // comment
            (Some('/'), _) => {
                while self.reader.peek() != Some('\n') && self.reader.peek().is_some() {
                    self.reader.next();
                }
                Ignore
            }
            _ => Div,
        }
    }

    /// parses all tokens that start with a %
    fn ambiguous_mod(&mut self) -> TokenType {
        use crate::lexer::TokenType::*;
        match (self.reader.peek(), self.reader.peek2()) {
            (Some('='), _) => {
                self.reader.next();
                ModEquals
            }
            _ => Mod,
        }
    }

    /// parses all tokens that start with <
    fn ambiguous_less(&mut self) -> TokenType {
        use crate::lexer::TokenType::*;
        match (self.reader.peek(), self.reader.peek2()) {
            (Some('<'), _) => {
                self.reader.next();
                DoubleLess
            }
            (Some('='), _) => {
                self.reader.next();
                LessEquals
            }
            _ => Less,
        }
    }

    /// parses all tokens that start with >
    fn ambiguous_greater(&mut self) -> TokenType {
        use crate::lexer::TokenType::*;
        match (self.reader.peek(), self.reader.peek2()) {
            (Some('>'), _) => {
                self.reader.next();
                DoubleGreater
            }
            (Some('='), _) => {
                self.reader.next();
                GreaterEquals
            }
            _ => Greater,
        }
    }

    /// parses all tokens that start with :
    fn ambiguous_colon(&mut self) -> TokenType {
        use crate::lexer::TokenType::*;
        match (self.reader.peek(), self.reader.peek2()) {
            (Some(':'), _) => {
                self.reader.next();
                ColonColon
            }
            (Some('='), _) => {
                self.reader.next();
                ColonEquals
            }
            _ => Colon,
        }
    }

    /// parses all tokens that start with a !
    fn ambiguous_bang(&mut self) -> TokenType {
        use crate::lexer::TokenType::*;
        match (self.reader.peek(), self.reader.peek2()) {
            (Some('='), _) => {
                self.reader.next();
                BangEquals
            }
            _ => Bang,
        }
    }

    /// parses all tokens that start with ?
    fn ambiguous_question(&mut self) -> TokenType {
        use crate::lexer::TokenType::*;
        match (self.reader.peek(), self.reader.peek2()) {
            (Some('?'), Some('=')) => {
                self.reader.next();
                self.reader.next();
                DoubleQuestionEqual
            }
            (Some('?'), _) => {
                self.reader.next();
                DoubleQuestion
            }
            _ => Question,
        }
    }

    /// parses all tokens that start with a =
    fn ambiguous_equal(&mut self) -> TokenType {
        use crate::lexer::TokenType::*;
        match (self.reader.peek(), self.reader.peek2()) {
            (Some('='), _) => {
                self.reader.next();
                EqualEqual
            }
            _ => Equal,
        }
    }

    /// parses all tokens that start with a .
    fn ambiguous_dot(&mut self) -> TokenType {
        use crate::lexer::TokenType::*;
        match (self.reader.peek(), self.reader.peek2()) {
            (Some('.'), Some('.')) => {
                self.reader.next();
                self.reader.next();
                Ellipses
            }
            _ => Dot,
        }
    }
}

impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        use TokenType::*;
        let c = self.reader.next()?;
        let token_type = match c {
            // match unambiguously single-character tokens
            '\'' => SingleQuote,
            '"' => DoubleQuote,
            '^' => Caret,
            '(' => LeftParen,
            ')' => RightParen,
            '[' => LeftBracket,
            ']' => RightBracket,
            '{' => LeftBrace,
            '}' => RightBrace,
            ',' => Comma,
            ';' => Semicolon,
            '|' => Pipe,

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
            ' ' | '\n' | '\r' | '\t' => Ignore,

            // TODO: string literals, requires modal lexing
            _ => error_handler::fatal_unreachable(),
        };
        return Some(Token {
            token_type,
            lexeme: self.reader.current,
            literal: None,
        });
    }
}
