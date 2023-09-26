#[derive(Debug, Clone, derive_more::Display)]
pub enum Value {
    Int(i64),
}

impl Value {
    pub fn neg(self) -> Option<Value> {
        match self {
            Value::Int(i) => Some(Value::Int(-i)),
            _ => todo!("type error handling"),
        }
    }

    pub fn pow(self, rhs: Value) -> Option<Value> {
        match (self, rhs) {
            (Value::Int(x), Value::Int(y)) => Some(Value::Int(x.pow(y as u32))),
            _ => todo!("type error handling"),
        }
    }

    pub fn mul(self, rhs: Value) -> Option<Value> {
        match (self, rhs) {
            (Value::Int(x), Value::Int(y)) => Some(Value::Int(x * y)),
            _ => todo!("type error handling"),
        }
    }

    pub fn div(self, rhs: Value) -> Option<Value> {
        match (self, rhs) {
            (Value::Int(x), Value::Int(y)) => {
                if y == 0 {
                    todo!("handle runtime errors");
                }
                Some(Value::Int(x / y))
            }
            _ => todo!("type error handling"),
        }
    }

    pub fn rem(self, rhs: Value) -> Option<Value> {
        match (self, rhs) {
            (Value::Int(x), Value::Int(y)) => {
                if y == 0 {
                    todo!("handle runtime errors");
                }
                Some(Value::Int(x % y))
            }
            _ => todo!("type error handling"),
        }
    }

    pub fn add(self, rhs: Value) -> Option<Value> {
        match (self, rhs) {
            (Value::Int(x), Value::Int(y)) => Some(Value::Int(x + y)),
            _ => todo!("type error handling"),
        }
    }

    pub fn sub(self, rhs: Value) -> Option<Value> {
        match (self, rhs) {
            (Value::Int(x), Value::Int(y)) => Some(Value::Int(x - y)),
            _ => todo!("type error handling"),
        }
    }
}
