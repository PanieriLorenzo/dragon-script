use crate::{
    eh::ErrorHandler,
    error_handler as eh,
    lexer::{Lexer, Token, TokenType as TT},
    lookahead::{lookahead, Lookahead},
    source::{Reader, SourceView},
};
use itertools::{multipeek, Itertools, MultiPeek, PeekingNext};
use std::{fmt::Display, iter::Filter, rc::Rc};
use sugars::boxed;

use anyhow::{Context, Error, Result};

mod ast;
pub use ast::*;

trait TokenStream = Iterator<Item = Token>;

///
/// # Method Naming Convention:
/// - `parse`: on fail doesn't consume tokens and returns error with context
/// - `match`: never consumes tokens, only advances lookahead, returns option
pub struct Parser {
    lx: Lookahead<Lexer>,
    eh: Rc<ErrorHandler>,
}

impl Parser {
    pub fn new(lx: Lexer, eh: &Rc<ErrorHandler>) -> Self {
        Self {
            lx: lookahead(lx),
            eh: eh.clone(),
        }
    }

    pub fn drop_all(&mut self) {
        self.eh.clone().unexpected_end_of_input();
        while let Some(_) = self.lx.next() {}
    }

    pub fn parse_expression(&mut self) -> Option<Expression> {
        self.parse_term()
    }

    pub fn parse_term(&mut self) -> Option<Expression> {
        let mut exp = self.parse_factor()?;
        while let Some(t) = self.match_one_of(&[TT::Plus, TT::Minus]) {
            self.lx.commit();
            let rhs = self.parse_factor()?;
            exp = Expression::BE(BinExpression {
                lhs: Box::new(exp),
                op: match t.token_type {
                    TT::Plus => BinOperator::Add,
                    TT::Minus => BinOperator::Sub,
                    _ => unreachable!(),
                },
                rhs: Box::new(rhs),
            });
        }
        Some(exp)
    }

    pub fn parse_factor(&mut self) -> Option<Expression> {
        let mut exp = self.parse_power()?;
        while let Some(t) = self.match_one_of(&[TT::Star, TT::Slash, TT::Percent]) {
            self.lx.commit();
            let rhs = self.parse_power()?;
            exp = Expression::BE(BinExpression {
                lhs: Box::new(exp),
                op: match t.token_type {
                    TT::Percent => BinOperator::Mod,
                    TT::Slash => BinOperator::Div,
                    TT::Star => BinOperator::Mul,
                    _ => unreachable!(),
                },
                rhs: Box::new(rhs),
            });
        }
        Some(exp)
    }

    pub fn parse_power(&mut self) -> Option<Expression> {
        let mut exp = self.parse_unary()?;
        while self.match_one(TT::Pow).is_some() {
            self.lx.commit();
            let rhs = self.parse_unary()?;
            exp = Expression::BE(BinExpression {
                lhs: Box::new(exp),
                op: BinOperator::Pow,
                rhs: Box::new(rhs),
            });
        }
        Some(exp)
    }

    pub fn parse_unary(&mut self) -> Option<Expression> {
        if self.match_one(TT::Minus).is_some() {
            self.lx.commit();
            let rhs = self.parse_unary()?;
            return Some(Expression::UE(UnExpression {
                op: UnOperator::Neg,
                rhs: Box::new(rhs),
            }));
        }
        self.parse_primary()
    }

    pub fn parse_primary(&mut self) -> Option<Expression> {
        if let Some(e) = self.match_int_literal() {
            self.lx.commit();
            return Some(e);
        }

        // TODO: cascade errors instead of reporting multiple times
        self.parse_grouping()
    }

    pub fn parse_grouping(&mut self) -> Option<Expression> {
        self.parse_one(TT::LeftParen)?;
        let e = self.parse_expression()?;
        self.parse_one(TT::RightParen)?;
        Some(e)
    }

    pub fn match_int_literal(&mut self) -> Option<Expression> {
        let t = self.match_one(TT::IntLit)?;
        Some(Expression::IntLiteral(t.lexeme))
    }

    pub fn parse_one(&mut self, tt: TT) -> Option<Token> {
        let t = self.match_one(tt);
        match (t, self.lx.current.clone()) {
            (None, None) => {
                self.eh.clone().unexpected_end_of_input();
                None
            }
            (None, Some(c)) => {
                self.eh
                    .clone()
                    .unexpected_token(c.lexeme, &[tt], c.token_type);
                None
            }
            (Some(t), _) => {
                self.lx.commit();
                Some(t)
            }
        }
    }

    fn parse_one_of(&mut self, tts: &[TT]) -> Option<Token> {
        let t = self.match_one_of(tts);
        match (t, self.lx.current.clone()) {
            (None, None) => {
                self.eh.clone().unexpected_end_of_input();
                None
            }
            (None, Some(c)) => {
                self.eh
                    .clone()
                    .unexpected_token(c.lexeme, tts, c.token_type);
                None
            }
            (Some(t), _) => {
                self.lx.commit();
                Some(t)
            }
        }
    }

    fn match_one(&mut self, tt: TT) -> Option<Token> {
        match self.lx.peek() {
            Some(ref t) if t.token_type == tt => Some(t.clone()),
            _ => {
                self.lx.reset();
                None
            }
        }
    }

    fn match_one_of(&mut self, tts: &[TT]) -> Option<Token> {
        for &tt in tts {
            if let o @ Some(_) = self.match_one(tt) {
                return o;
            }
        }
        None
    }
}

// impl std::fmt::Display for ParseTreeNodeType {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let Self::Token(t) = self;
//         write!(f, "{}", t)?;
//         Ok(())
//     }
// }
