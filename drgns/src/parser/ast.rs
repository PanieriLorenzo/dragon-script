use std::fmt::Display;

use crate::values::Value;

#[derive(Debug, Clone, derive_more::Display)]
pub enum Expression {
    BE(BinExpression),
    UE(UnExpression),
    LE(LitExpression),
}

impl Expression {
    /// walk the expression tree, applying the visitor to each node. Nodes are
    /// visited in postfix order, so evaluation can be implemented with a simple
    /// stack machine
    pub fn walk(&self, v: &mut impl Visitor<()>) {
        match &self {
            Self::BE(be) => be.walk(v),
            Self::UE(ue) => ue.walk(v),
            Self::LE(le) => le.walk(v),
        }
        v.visit_expression(self);
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
        self.rhs.walk(v);
        self.lhs.walk(v);
        self.op.walk(v);
        v.visit_bin_expression(self);
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
    Mul,
    Div,
    Mod,
    Add,
    Sub,
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
            Self::Mul => write!(f, "*"),
            Self::Div => write!(f, "/"),
            Self::Mod => write!(f, "%"),
            Self::Add => write!(f, "+"),
            Self::Sub => write!(f, "-"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UnExpression {
    pub op: UnOperator,
    pub rhs: Box<Expression>,
}

impl UnExpression {
    fn walk(&self, v: &mut impl Visitor<()>) {
        self.rhs.walk(v);
        self.op.walk(v);
        v.visit_un_expression(self);
    }
}

impl Display for UnExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {})", self.op, self.rhs)
    }
}

#[derive(Debug, Clone)]
pub enum UnOperator {
    Neg,
}

impl UnOperator {
    fn walk(&self, v: &mut impl Visitor<()>) {
        v.visit_un_operator(self);
    }
}

impl Display for UnOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Neg => write!(f, "-"),
        }
    }
}

#[derive(Debug, Clone, derive_more::Display)]
pub struct LitExpression(pub Value);

impl LitExpression {
    fn walk(&self, v: &mut impl Visitor<()>) {
        v.visit_lit_expression(self)
    }
}

pub trait Visitor<T> {
    fn visit_expression(&mut self, e: &Expression) -> T;
    fn visit_bin_expression(&mut self, be: &BinExpression) -> T;
    fn visit_bin_operator(&mut self, bo: &BinOperator) -> T;
    fn visit_un_expression(&mut self, ue: &UnExpression) -> T;
    fn visit_un_operator(&mut self, uo: &UnOperator) -> T;
    fn visit_lit_expression(&mut self, le: &LitExpression) -> T;
}
