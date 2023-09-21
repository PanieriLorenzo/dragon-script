use std::fmt::Display;

use crate::arena::Span;

#[derive(Debug, Clone)]
pub enum Expression {
    BinExpression(BinExpression),
    IntLiteral(Span),
}

impl Expression {
    fn walk(&self, v: &mut impl Visitor<()>) {
        v.visit_expression(self);
        match &self {
            Self::BinExpression(be) => be.walk(v),
            Self::IntLiteral(_) => {}
        }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BinExpression(be) => {
                write!(f, "{}", be)
            }
            Self::IntLiteral(i) => write!(f, "{}", i.into_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BinExpression {
    pub lhs: Box<Expression>,
    pub op: BinOperator,
    pub rhs: Box<Expression>,
}

impl BinExpression {
    fn walk(&self, v: &mut impl Visitor<()>) {
        v.visit_bin_expression(self);
        self.lhs.walk(v);
        self.op.walk(v);
        self.rhs.walk(v);
    }
}

impl Display for BinExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {} {})", self.op, self.lhs, self.rhs)
    }
}

#[derive(Debug, Clone)]
pub enum BinOperator {
    Pow,
}

impl BinOperator {
    fn walk(&self, v: &mut impl Visitor<()>) {
        v.visit_bin_operator(self);
    }
}

impl Display for BinOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pow => write!(f, "**"),
        }
    }
}

pub trait Visitor<T> {
    fn visit_expression(&mut self, e: &Expression) -> T;
    fn visit_bin_expression(&mut self, be: &BinExpression) -> T;
    fn visit_bin_operator(&mut self, bo: &BinOperator) -> T;
}
