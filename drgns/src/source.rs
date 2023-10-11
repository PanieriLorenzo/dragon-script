use std::{fs::read_to_string, io::Result, ops::Range, rc};

mod arena;
pub use arena::*;
mod view;
pub use view::*;
mod string;
pub use string::*;
mod reader;
pub use reader::*;

/// A piece of source code, either a file or a REPL logical line
pub struct Source {
    name: Option<String>,
    src: Vec<char>,
}

impl Source {
    pub fn from_path(p: &str) -> Result<Self> {
        let mut ret = Self {
            name: Some(p.to_owned()),
            src: vec![],
        };
        ret.src = read_to_string(p)?.chars().collect();
        Ok(ret)
    }

    pub fn from_string(s: String) -> Self {
        Self {
            name: None,
            src: s.chars().collect(),
        }
    }
}
