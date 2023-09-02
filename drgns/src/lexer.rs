use std::{
    cell::OnceCell,
    collections::HashMap,
    sync::{OnceLock, RwLock},
};

use append_only_vec::AppendOnlyVec;
use clap::error;
use smallvec::SmallVec;

use crate::{
    arena::{Reader, Span},
    data,
    data::PrimitiveValue,
    error_handler,
};

#[derive(Debug, Clone, Copy, PartialEq)]
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
    CloseInterpolation,
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
    StringPart,
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

fn init_keywords() -> &'static RwLock<HashMap<&'static str, TokenType>> {
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
    })
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

/// lexer modes let us deal with things like nested string interpolations and
/// unpaired delimiters
enum LexerMode {
    // starts in normal mode
    Normal,

    // a block surronded by parens
    ParenBlock,

    // a block surrounded by brackets
    BracketBlock,

    // a block surrounded by braces
    BraceBlock,

    // the body of a string literal, which may contain escae sequences and
    // string interpolations
    String,

    // very similar to normal mode, but deals with unmatched closing braces as
    // endings to interpolations
    Interpolation,
}

pub struct Lexer {
    reader: Reader,
    // the idea of using SmallVec is that in 99.99% of cases, you'll never nest
    // your string interpolations more than 32 layers. Using SmallStack is easier
    // than implementing my own stack-allocated stack with safe overflows. One
    // less error to keep track of.
    mode_stack: SmallVec<[LexerMode; 32]>,
}

impl Lexer {
    pub fn new(reader: Reader) -> Self {
        Self {
            reader,
            mode_stack: vec![LexerMode::Normal].into(),
        }
    }

    pub fn delim_depth(&self) -> usize {
        self.mode_stack.len()
    }

    /// parses all tokens that start with +
    fn ambiguous_plus(&mut self) -> (TokenType, Option<PrimitiveValue>) {
        use crate::lexer::TokenType as T;
        match (self.reader.peek(), self.reader.peek2()) {
            (Some('+'), Some('=')) => {
                self.reader.next();
                self.reader.next();
                (T::PlusPlusEquals, None)
            }
            (Some('+'), _) => {
                self.reader.next();
                (T::PlusPlus, None)
            }
            (Some('='), _) => {
                self.reader.next();
                (T::PlusEquals, None)
            }
            _ => (T::Plus, None),
        }
    }

    /// parses all tokens that start with *
    fn ambiguous_mul(&mut self) -> (TokenType, Option<PrimitiveValue>) {
        use crate::lexer::TokenType as T;
        match (self.reader.peek(), self.reader.peek2()) {
            (Some('*'), Some('=')) => {
                self.reader.next();
                self.reader.next();
                (T::MulMulEquals, None)
            }
            (Some('*'), _) => {
                self.reader.next();
                (T::MulMul, None)
            }
            (Some('='), _) => {
                self.reader.next();
                (T::MulEquals, None)
            }
            _ => (T::Mul, None),
        }
    }

    /// parses all tokens that start with -
    fn ambiguous_minus(&mut self) -> (TokenType, Option<PrimitiveValue>) {
        use crate::lexer::TokenType as T;
        match (self.reader.peek(), self.reader.peek2()) {
            (Some('='), _) => {
                self.reader.next();
                (T::MinusEquals, None)
            }
            (Some('>'), _) => {
                self.reader.next();
                (T::Arrow, None)
            }
            _ => (T::Minus, None),
        }
    }

    /// parses all tokens that start with a /
    fn ambiguous_slash(&mut self) -> (TokenType, Option<PrimitiveValue>) {
        use crate::lexer::TokenType as T;
        match (self.reader.peek(), self.reader.peek2()) {
            (Some('='), _) => {
                self.reader.next();
                (T::DivEquals, None)
            }
            // comment
            (Some('/'), _) => {
                while self.reader.peek() != Some('\n') && self.reader.peek().is_some() {
                    self.reader.next();
                }
                (T::Ignore, None)
            }
            _ => (T::Div, None),
        }
    }

    /// parses all tokens that start with a %
    fn ambiguous_mod(&mut self) -> (TokenType, Option<PrimitiveValue>) {
        use crate::lexer::TokenType as T;
        match (self.reader.peek(), self.reader.peek2()) {
            (Some('='), _) => {
                self.reader.next();
                (T::ModEquals, None)
            }
            _ => (T::Mod, None),
        }
    }

    /// parses all tokens that start with <
    fn ambiguous_less(&mut self) -> (TokenType, Option<PrimitiveValue>) {
        use crate::lexer::TokenType as T;
        match (self.reader.peek(), self.reader.peek2()) {
            (Some('<'), _) => {
                self.reader.next();
                (T::DoubleLess, None)
            }
            (Some('='), _) => {
                self.reader.next();
                (T::LessEquals, None)
            }
            _ => (T::Less, None),
        }
    }

    /// parses all tokens that start with >
    fn ambiguous_greater(&mut self) -> (TokenType, Option<PrimitiveValue>) {
        use crate::lexer::TokenType as T;
        match (self.reader.peek(), self.reader.peek2()) {
            (Some('>'), _) => {
                self.reader.next();
                (T::DoubleGreater, None)
            }
            (Some('='), _) => {
                self.reader.next();
                (T::GreaterEquals, None)
            }
            _ => (T::Greater, None),
        }
    }

    /// parses all tokens that start with :
    fn ambiguous_colon(&mut self) -> (TokenType, Option<PrimitiveValue>) {
        use crate::lexer::TokenType as T;
        match (self.reader.peek(), self.reader.peek2()) {
            (Some(':'), _) => {
                self.reader.next();
                (T::ColonColon, None)
            }
            (Some('='), _) => {
                self.reader.next();
                (T::ColonEquals, None)
            }
            _ => (T::Colon, None),
        }
    }

    /// parses all tokens that start with a !
    fn ambiguous_bang(&mut self) -> (TokenType, Option<PrimitiveValue>) {
        use crate::lexer::TokenType as T;
        match (self.reader.peek(), self.reader.peek2()) {
            (Some('='), _) => {
                self.reader.next();
                (T::BangEquals, None)
            }
            _ => (T::Bang, None),
        }
    }

    /// parses all tokens that start with ?
    fn ambiguous_question(&mut self) -> (TokenType, Option<PrimitiveValue>) {
        use crate::lexer::TokenType as T;
        match (self.reader.peek(), self.reader.peek2()) {
            (Some('?'), Some('=')) => {
                self.reader.next();
                self.reader.next();
                (T::DoubleQuestionEqual, None)
            }
            (Some('?'), _) => {
                self.reader.next();
                (T::DoubleQuestion, None)
            }
            _ => (T::Question, None),
        }
    }

    /// parses all tokens that start with a =
    fn ambiguous_equal(&mut self) -> (TokenType, Option<PrimitiveValue>) {
        use crate::lexer::TokenType as T;
        match (self.reader.peek(), self.reader.peek2()) {
            (Some('='), _) => {
                self.reader.next();
                (T::EqualEqual, None)
            }
            _ => (T::Equal, None),
        }
    }

    /// parses all tokens that start with a .
    fn ambiguous_dot(&mut self) -> (TokenType, Option<PrimitiveValue>) {
        use crate::lexer::TokenType as T;
        match (self.reader.peek(), self.reader.peek2()) {
            (Some('.'), Some('.')) => {
                self.reader.next();
                self.reader.next();
                (T::Ellipses, None)
            }
            _ => (T::Dot, None),
        }
    }

    fn ambiguous_dollar(&mut self) -> (TokenType, Option<PrimitiveValue>) {
        use crate::lexer::TokenType as T;
        match (self.reader.peek(), self.reader.peek2()) {
            (Some('{'), _) => {
                self.reader.next();
                self.mode_stack.push(LexerMode::Interpolation);
                (T::DollarBrace, None)
            }
            _ => (T::StringPart, None),
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

    fn lex_identifier(&mut self) -> (TokenType, Option<PrimitiveValue>) {
        while self
            .reader
            .peek()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            self.reader.next();
        }

        let text = self.reader.current.to_string();
        if let Some(type_) = init_keywords()
            .read()
            .unwrap_or_else(|_| error_handler::fatal_generic("poisoned lock"))
            .get(&text as &str)
        {
            (*type_, None)
        } else {
            (TokenType::Identifier, None)
        }
    }

    fn lex_raw_string(&mut self) -> (TokenType, Option<PrimitiveValue>) {
        loop {
            if self.reader.peek().is_none() {
                error_handler::err_unclosed_delimiter('\'');
                break;
            }

            if self.reader.peek().is_some_and(|c| c == '\'') {
                self.reader.next();
                break;
            }
            self.reader.next();
        }

        (
            TokenType::RawString,
            Some(PrimitiveValue::String(self.reader.current.to_string())),
        )
    }

    fn lex_string_part(&mut self) -> (TokenType, Option<PrimitiveValue>) {
        // immediately handles escaped characters, instead of using separate tokens
        let mut literal: Vec<char> = vec![self
            .reader
            .current
            .to_string()
            .chars()
            .next()
            .unwrap_or_else(|| error_handler::fatal_unreachable())];
        loop {
            match (self.reader.peek(), self.reader.peek2()) {
                (None, _) | (Some('"'), _) => break,
                // TODO: handle special escape sequences like `\u1234`
                (Some('\\'), Some(c)) => {
                    self.reader.next();
                    self.reader.next();
                    literal.push(c);
                }
                (Some(c), _) => {
                    self.reader.next();
                    literal.push(c);
                }
            }
        }
        (
            TokenType::StringPart,
            Some(PrimitiveValue::String(literal.iter().collect())),
        )
    }

    fn normal_mode_next(&mut self) -> Option<Token> {
        use TokenType as T;
        let c = self.reader.next()?;
        let (token_type, literal) = match c {
            // match unambiguously single-character tokens
            // '\'' => (T::SingleQuote, None),
            // '"' => (T::DoubleQuote, None),
            '^' => (T::Caret, None),
            '(' => {
                self.mode_stack.push(LexerMode::ParenBlock);
                (T::LeftParen, None)
            }
            ')' => {
                // we never expect to reach a closing parens in normal mode,
                // because we can only see a closing parens from within a ParensBlock
                error_handler::err_unmatched_delimiter(')');
                (T::Ignore, None)
            }
            '[' => {
                self.mode_stack.push(LexerMode::BracketBlock);
                (T::LeftBracket, None)
            }
            ']' => {
                error_handler::err_unmatched_delimiter(']');
                (T::Ignore, None)
            }
            '{' => {
                self.mode_stack.push(LexerMode::BraceBlock);
                (T::LeftBrace, None)
            }
            '}' => {
                error_handler::err_unmatched_delimiter('}');
                (T::Ignore, None)
            }
            ',' => (T::Comma, None),
            ';' => (T::Semicolon, None),
            '|' => (T::Pipe, None),

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
            ' ' | '\n' | '\r' | '\t' => (T::Ignore, None),

            // raw tring literals
            '\'' => self.lex_raw_string(),

            // string literals, require modal lexing
            '\"' => {
                self.mode_stack.push(LexerMode::String);
                (T::DoubleQuote, None)
            }

            // numbers
            c if c.is_ascii_digit() => self.lex_number_literal(),
            c if c.is_ascii_alphabetic() || c == '_' => self.lex_identifier(),

            _ => {
                error_handler::err_unexpected_character(c);
                (T::Unknown, None)
            }
        };
        Some(Token {
            token_type,
            lexeme: self.reader.advance_tail(),
            literal: literal,
        })
    }

    /// proxy mode for normal mode that intercepts closing braces to terminate
    /// interpolations
    fn interpolation_mode_next(&mut self) -> Option<Token> {
        use TokenType as T;
        match self.reader.peek() {
            Some('}') => {
                self.reader.next();
                self.mode_stack
                    .pop()
                    .unwrap_or_else(|| error_handler::fatal_unreachable());
                Some(Token {
                    token_type: T::RightBrace,
                    lexeme: self.reader.advance_tail(),
                    literal: None,
                })
            }
            _ => self.normal_mode_next(),
        }
    }

    /// proxy mode for normal mode that intercepts closing parens
    fn paren_block_mode_next(&mut self) -> Option<Token> {
        use TokenType as T;
        match self.reader.peek() {
            Some(')') => {
                self.reader.next();
                self.mode_stack
                    .pop()
                    .unwrap_or_else(|| error_handler::fatal_unreachable());
                Some(Token {
                    token_type: T::RightParen,
                    lexeme: self.reader.advance_tail(),
                    literal: None,
                })
            }
            _ => self.normal_mode_next(),
        }
    }

    /// proxy mode for normal mode that intercepts closing bracket
    fn bracket_block_mode_next(&mut self) -> Option<Token> {
        use TokenType as T;
        match self.reader.peek() {
            Some(']') => {
                self.reader.next();
                self.mode_stack
                    .pop()
                    .unwrap_or_else(|| error_handler::fatal_unreachable());
                Some(Token {
                    token_type: T::RightBracket,
                    lexeme: self.reader.advance_tail(),
                    literal: None,
                })
            }
            _ => self.normal_mode_next(),
        }
    }

    /// proxy mode for normal mode that intercepts closing brace
    fn brace_block_mode_next(&mut self) -> Option<Token> {
        use TokenType as T;
        match self.reader.peek() {
            Some('}') => {
                self.reader.next();
                self.mode_stack
                    .pop()
                    .unwrap_or_else(|| error_handler::fatal_unreachable());
                Some(Token {
                    token_type: T::RightBracket,
                    lexeme: self.reader.advance_tail(),
                    literal: None,
                })
            }
            _ => self.normal_mode_next(),
        }
    }

    fn string_mode_next(&mut self) -> Option<Token> {
        use TokenType as T;
        let c = self.reader.next()?;
        let (token_type, literal) = match c {
            '$' => self.ambiguous_dollar(),
            '"' => {
                self.mode_stack.pop();
                (T::DoubleQuote, None)
            }
            _ => self.lex_string_part(),
        };
        Some(Token {
            token_type,
            lexeme: self.reader.advance_tail(),
            literal: literal,
        })
    }
}

impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        use LexerMode as LM;
        match self.mode_stack.last() {
            None => None,
            Some(LM::Normal) => self.normal_mode_next(),
            Some(LM::Interpolation) => self.interpolation_mode_next(),
            Some(LM::String) => self.string_mode_next(),
            Some(LM::ParenBlock) => self.paren_block_mode_next(),
            Some(LM::BracketBlock) => self.bracket_block_mode_next(),
            Some(LM::BraceBlock) => self.brace_block_mode_next(),
        }
    }
}
