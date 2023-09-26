use std::{
    collections::HashMap,
    fmt::Display,
    rc::Rc,
    sync::{OnceLock, RwLock, Weak},
};

use anyhow::Error;
use smallvec::SmallVec;
use strum_macros::EnumIter;

use crate::{
    assert_unreachable,
    eh::ErrorHandler,
    error_handler as eh, internal_error,
    source::{Reader, SourceView},
};

#[cfg(test)]
mod test_utils;

#[cfg(test)]
mod test;

#[derive(Debug, Clone, Copy, PartialEq, EnumIter)]
pub enum TokenType {
    // unambiguously single-character tokens
    Semicolon,
    LeftParen,
    RightParen,
    Comma,
    Plus,
    Percent,
    Minus,

    // one or more chars
    Slash, // or comment
    Star,
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
    pub lexeme: SourceView,
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

#[derive(Clone)]
pub struct Lexer {
    reader: Reader,
    eh: Rc<ErrorHandler>,
}

impl Lexer {
    pub fn new(reader: Reader, eh: &Rc<ErrorHandler>) -> Self {
        Self {
            reader,
            eh: eh.clone(),
        }
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
            _ => T::Slash,
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

        let text = self.reader.current.clone().into_string();
        if let Some(type_) = init_keywords()
            .read()
            .unwrap_or_else(|_| internal_error!("poisoned lock"))
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
            ';' => TT::Semicolon,
            ')' => TT::RightParen,
            '(' => TT::LeftParen,
            ',' => TT::Comma,
            '+' => TT::Plus,
            '%' => TT::Percent,
            '-' => TT::Minus,

            // one or more chars
            '/' => self.lex_div_or_comment(), // or comment
            '*' => self
                .lex_postfixes(&[(&[Some('*')], TT::Pow), (&[], TT::Star)])
                .unwrap_or_else(|| assert_unreachable!()),

            // two character
            ':' => self
                .lex_postfixes(&[(&[Some('=')], TT::ColonEquals)])
                .unwrap_or_else(|| {
                    self.eh
                        .clone()
                        .unexpected_char(self.reader.current.clone(), c);
                    TT::Unknown
                }),

            // ignore whitespace
            ' ' | '\n' | '\r' | '\t' => TT::Ignore,

            // numbers
            c if c.is_ascii_digit() => self.lex_number_literal(),

            // literals
            c if c.is_ascii_alphabetic() || c == '_' => self.lex_identifier(),

            _ => {
                log::trace!("unmatched char");
                // self.eh
                //     .clone()
                //     .unexpected_char(self.reader.current.clone(), c);
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
        use TokenType as TT;
        let ot = self.normal_mode_next();
        if ot.clone().is_some_and(|t| t.token_type == TT::Ignore) {
            return self.next();
        }
        ot
    }
}
