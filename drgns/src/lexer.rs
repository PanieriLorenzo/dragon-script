use std::{
    collections::HashMap,
    sync::{OnceLock, RwLock},
};

use append_only_vec::AppendOnlyVec;

use crate::{arena::Span, data};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // unambiguously single-character tokens
    SingleQuote,
    DoubleQuote,
    Caret,
    LeftParent,
    RightParent,
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
    DoubleGreaterEquals,
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
    None,
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
                ("none", TokenType::None),
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

struct LexerContext {
    tokens: Vec<Token>,
    start: Span,
    current: Span,
}
