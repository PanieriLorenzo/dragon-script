use std::{
    collections::HashMap,
    fmt::Display,
    sync::{OnceLock, RwLock},
};

use smallvec::SmallVec;
use strum_macros::EnumIter;

use crate::{
    arena::{Reader, Span},
    error_handler as eh,
};

#[cfg(test)]
mod test_utils;

#[cfg(test)]
mod test;

#[derive(Debug, Clone, Copy, PartialEq, EnumIter)]
pub enum TokenType {
    // unambiguously single-character tokens
    LeftParen,
    RightParen,
    Comma,
    Plus,
    Mod,
    Minus,

    // one or more chars
    Div, // or comment
    Mul,
    Pow,

    // two character
    ColonEquals,

    // literals
    Identifier,
    IntLit,

    // Keywords
    Exit,

    // whitespace, comments and already handled tokens
    Ignore,

    // unrecognized tokens
    Unknown,
}

type OnceMap<K, V> = OnceLock<RwLock<HashMap<K, V>>>;

static KEYWORDS: OnceMap<&'static str, TokenType> = OnceLock::new();

fn init_keywords() -> &'static RwLock<HashMap<&'static str, TokenType>> {
    KEYWORDS.get_or_init(|| RwLock::new([("exit", TokenType::Exit)].iter().cloned().collect()))
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: Span,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.token_type, self.lexeme)
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

    /// lex a group of tokens that share a common prefix, for example
    /// <= and <. Provide a list of mappings from postfixes to tokens, they
    /// are matched in order, so the most specific should be first.
    ///
    /// Example mapping:
    /// ```txt
    /// [=]     => <=
    /// []      => <
    /// ```
    fn lex_postfixes(&mut self, mappings: &[(&[Option<char>], TokenType)]) -> Option<TokenType> {
        mappings.iter().find_map(|(cs, tt)| {
            cs.iter()
                .enumerate()
                .all(|(i, &c)| self.reader.peek_n(i) == c)
                .then(|| {
                    (0..cs.len()).for_each(|_| {
                        self.reader.next();
                    });
                    *tt
                })
        })
    }

    /// parses all tokens that start with a /
    fn lex_div_or_comment(&mut self) -> TokenType {
        use crate::lexer::TokenType as T;
        match self.reader.peek() {
            // comment
            Some('/') => {
                while self.reader.peek() != Some('\n') && self.reader.peek().is_some() {
                    self.reader.next();
                }
                T::Ignore
            }
            _ => T::Div,
        }
    }

    fn lex_number_literal(&mut self) -> TokenType {
        // helper for matching digit or digit separator, e.g. 123_456_789
        let is_digit_or_sep = |c: char| c.is_ascii_digit() || c == '_';

        // match integer part
        while self.reader.peek().is_some_and(is_digit_or_sep) {
            self.reader.next();
        }

        TokenType::IntLit
    }

    fn lex_identifier(&mut self) -> TokenType {
        while self
            .reader
            .peek()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            self.reader.next();
        }

        let text = self.reader.current.into_string();
        if let Some(type_) = init_keywords()
            .read()
            .unwrap_or_else(|_| eh::fatal_generic("poisoned lock"))
            .get(text.as_str())
        {
            *type_
        } else {
            TokenType::Identifier
        }
    }

    fn normal_mode_next(&mut self) -> Option<Token> {
        use TokenType as TT;
        let c = self.reader.next()?;
        let token_type = match c {
            // unambiguously single-character tokens
            ')' => TT::LeftParen,
            '(' => TT::RightParen,
            ',' => TT::Comma,
            '+' => TT::Plus,
            '%' => TT::Mod,
            '-' => TT::Minus,

            // one or more chars
            '/' => self.lex_div_or_comment(), // or comment
            '*' => self
                .lex_postfixes(&[(&[Some('*')], TT::Pow), (&[], TT::Mul)])
                .unwrap_or_else(|| eh::fatal_unreachable()),

            // two character
            ':' => self
                .lex_postfixes(&[(&[Some('=')], TT::ColonEquals)])
                .unwrap_or_else(|| {
                    eh::err_unexpected_character(c);
                    TT::Unknown
                }),

            // ignore whitespace
            ' ' | '\n' | '\r' | '\t' => TT::Ignore,

            // numbers
            c if c.is_ascii_digit() => self.lex_number_literal(),

            // literals
            c if c.is_ascii_alphabetic() || c == '_' => self.lex_identifier(),

            _ => {
                eh::err_unexpected_character(c);
                TT::Unknown
            }
        };
        Some(Token {
            token_type,
            lexeme: self.reader.advance_tail(),
        })
    }
}

impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        use LexerMode as LM;
        use TokenType as TT;
        loop {
            let ot = match self.mode_stack.last() {
                None => None,
                Some(LM::Normal) => self.normal_mode_next(),
            };
            if ot.as_ref().is_some_and(|t| t.token_type != TT::Ignore) {
                return ot;
            }
        }
    }
}
