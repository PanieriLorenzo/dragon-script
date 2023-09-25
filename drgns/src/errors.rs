use miette::{Diagnostic, SourceSpan};
use std::io;
use thiserror::Error;

use crate::source::SourceView;

#[derive(Error, Debug)]
pub enum DragonError {
    #[error("IO error: {1}")]
    IOError(io::Error, String),

    #[error("Lexer error: {1}")]
    LexerError(SourceView, String),

    #[error("Parser error: {1}")]
    ParserError(SourceView, String),
}

impl DragonError {
    pub fn io_error(original: io::Error, context: String) -> Self {
        Self::IOError(original, context)
    }

    pub fn lexer_error(span: SourceView, context: String) -> Self {
        Self::LexerError(span, context)
    }

    pub fn parser_error(span: SourceView, context: String) -> Self {
        Self::ParserError(span, context)
    }
}

impl From<io::Error> for DragonError {
    fn from(value: io::Error) -> Self {
        let s = value.to_string();
        Self::IOError(value, s)
    }
}
