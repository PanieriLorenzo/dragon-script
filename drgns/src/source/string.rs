use std::rc;

use super::*;

#[derive(Clone)]
pub struct SourceString {
    source: rc::Weak<Source>,
    pos: Range<usize>,
}
