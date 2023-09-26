use crate::{
    parser::{LitExpression, Visitor},
    values::Value,
};

#[derive(Debug)]
pub struct ExpressionEval(Vec<Value>);

impl ExpressionEval {
    pub fn new() -> Self {
        Self(vec![])
    }
}

impl Visitor<()> for ExpressionEval {
    fn visit_expression(&mut self, _: &crate::parser::Expression) {}

    fn visit_bin_expression(&mut self, _be: &crate::parser::BinExpression) {}

    fn visit_bin_operator(&mut self, bo: &crate::parser::BinOperator) {
        match bo {
            crate::parser::BinOperator::Pow => {
                let x = self.0.pop().unwrap();
                let y = self.0.pop().unwrap();
                self.0.push(x.pow(y).unwrap());
            }
            crate::parser::BinOperator::Mul => {
                let x = self.0.pop().unwrap();
                let y = self.0.pop().unwrap();
                self.0.push(x.mul(y).unwrap());
            }
            crate::parser::BinOperator::Div => {
                let x = self.0.pop().unwrap();
                let y = self.0.pop().unwrap();
                self.0.push(x.div(y).unwrap());
            }
            crate::parser::BinOperator::Mod => {
                let x = self.0.pop().unwrap();
                let y = self.0.pop().unwrap();
                self.0.push(x.rem(y).unwrap());
            }
            crate::parser::BinOperator::Add => {
                let x = self.0.pop().unwrap();
                let y = self.0.pop().unwrap();
                self.0.push(x.add(y).unwrap());
            }
            crate::parser::BinOperator::Sub => {
                let x = self.0.pop().unwrap();
                let y = self.0.pop().unwrap();
                self.0.push(x.sub(y).unwrap());
            }
        }
    }

    fn visit_un_expression(&mut self, _ue: &crate::parser::UnExpression) {}

    fn visit_un_operator(&mut self, uo: &crate::parser::UnOperator) {
        match uo {
            crate::parser::UnOperator::Neg => {
                let x = self.0.pop().unwrap();
                self.0.push(x.neg().unwrap());
            }
        }
    }

    fn visit_lit_expression(&mut self, le: &crate::parser::LitExpression) {
        match le {
            LitExpression(v) => self.0.push(v.clone()),
        }
    }
}
