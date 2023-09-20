use crate::{
    error_handler as eh,
    lexer::{Lexer, Token, TokenType as TT},
};
use itertools::{multipeek, Itertools, MultiPeek};
use std::fmt::Display;

pub struct Parser {
    pub lexer: MultiPeek<Lexer>,
}

#[derive(Debug)]
pub enum ParseTreeNode {
    Token(Token),
}

impl std::fmt::Display for ParseTreeNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self::Token(t) = self;
        write!(f, "{}", t)?;
        Ok(())
    }
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        Self {
            lexer: lexer.multipeek(),
        }
    }

    /// check if the next token matches a token type
    fn match_one(&mut self, tt: TT) -> bool {
        let ret = self.lexer.peek().is_some_and(|x| x.token_type == tt);
        self.lexer.reset_peek();
        ret
    }

    /// check if the next token matches any token type from a list
    fn match_any(&mut self, tts: &[TT]) -> bool {
        let next_t = match self.lexer.peek() {
            Some(t) => t,
            None => return false,
        };
        tts.iter().any(|tt| next_t.token_type == *tt)
    }
}

impl Iterator for Parser {
    type Item = ParseTreeNode;

    fn next(&mut self) -> Option<Self::Item> {
        Some(ParseTreeNode::Token(self.lexer.next()?))
    }
}
