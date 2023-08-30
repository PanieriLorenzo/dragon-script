/// model data in the interpreter state

#[derive(Debug, Clone, PartialEq)]
pub enum PrimitiveValue {
    None,
    True,
    False,
    Int(u64),
    Float(f64),
    String(String),
}

impl ToString for PrimitiveValue {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}
