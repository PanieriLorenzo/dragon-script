use crate::{
    arena::Reader,
    error_handler as eh,
    lexer::{Lexer, Token, TokenType as TT},
};
use itertools::{multipeek, Itertools, MultiPeek, PeekingNext};
use std::{fmt::Display, iter::Filter};
use sugars::boxed;

trait TokenStream = Iterator<Item = Token>;
trait Matcher = FnMut(Parser) -> Option<ParseTreeNodeType>;

///
/// # Method Naming Convention:
/// - `parse`: consumes tokens from the stream, returns a result with the parsed node
/// - `drop`: consumes tokens from the stream, returns an empty result
/// - `match`: does not consume the tokens, peek head is reset if it doesn't match, returns optional parsed node
/// - `check`: does not consume the tokens, always advances head, returns bool
/// - `lookahead`: does not consume the tokens, always resets peek head, returns a boolean
pub struct Parser(MultiPeek<Lexer>);

impl Parser {
    pub fn new(lx: Lexer) -> Self {
        Self(lx.multipeek())
    }

    /*
    UnExpression
    = Minus, Factor
    ;
     */
    pub fn match_un_expression(&mut self) -> Option<ParseTreeNodeType> {
        self.match_one(TT::Minus)?;
        let factor = self.match_int_literal()?;
        Some(ParseTreeNodeType::UnExpression {
            op: boxed!(ParseTreeNodeType::Neg),
            factor: boxed!(factor),
        })
    }

    /*
    Factor
    = Identifier
    | IntLiteral
    | LeftParen, Expression, RightParen
    ;
     */
    pub fn match_int_literal(&mut self) -> Option<ParseTreeNodeType> {
        let t = self.match_one(TT::IntLit)?;
        Some(ParseTreeNodeType::IntLiteral)
    }

    pub fn match_one(&mut self, tt: TT) -> Option<Token> {
        match self.0.peek() {
            Some(&ref t) if t.token_type == tt => Some(t.clone()),
            _ => {
                self.0.reset_peek();
                None
            }
        }
    }
}

#[derive(Debug)]
pub enum ParseTreeNodeType {
    Neg,
    UnExpression {
        op: Box<ParseTreeNodeType>,
        factor: Box<ParseTreeNodeType>,
    },
    IntLiteral,
}

// impl std::fmt::Display for ParseTreeNodeType {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let Self::Token(t) = self;
//         write!(f, "{}", t)?;
//         Ok(())
//     }
// }
