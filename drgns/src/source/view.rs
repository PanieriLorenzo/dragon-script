use std::fmt::{Debug, Display};
use std::ops::Add;
use std::sync::{Arc, RwLockReadGuard, Weak};

use miette::{SourceCode, SourceSpan};

use crate::assert_pre_condition;

use super::arena::SourceArena;

#[derive(Clone)]
pub struct SourceView {
    pub arena: Weak<SourceArena>,
    pub span: SourceSpan,
}

impl SourceCode for SourceView {
    fn read_span<'a>(
        &'a self,
        span: &miette::SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<Box<dyn miette::SpanContents<'a> + 'a>, miette::MietteError> {
        todo!()
    }
}

impl SourceView {
    pub fn from_arena(arena: &Arc<SourceArena>) -> Self {
        Self {
            arena: Arc::downgrade(arena),
            span: SourceSpan::from((0, arena.len())),
        }
    }

    pub fn grow(&mut self) {
        self.span = SourceSpan::from((self.span.offset(), self.span.len() + 1));
    }

    /// Increment offset up to current end, and reset length to 0
    pub fn pull_tail(&mut self) {
        self.span = SourceSpan::from((self.end(), 0));
    }

    pub fn start(&self) -> usize {
        self.span.offset()
    }

    pub fn len(&self) -> usize {
        self.span.len()
    }

    pub fn end(&self) -> usize {
        self.span.offset() + self.span.len()
    }

    pub fn into_string(self) -> String {
        self.arena
            .upgrade()
            .unwrap()
            .inner()
            .iter()
            .skip(self.start())
            .take(self.len())
            .collect()
    }

    /// get a char from the span given index
    pub fn get(&self, i: usize) -> char {
        // pre-condition: index in range
        crate::assert_pre_condition!(i < self.len());

        self.arena.upgrade().unwrap().get(i - self.len()).unwrap()
    }

    /// get a char from the span indexing from the end
    pub fn get_back(&self, i: usize) -> char {
        // pre-condition: index in range
        crate::assert_pre_condition!(self.start() + self.len() - i < self.len());

        self.arena
            .upgrade()
            .unwrap()
            .get(self.start() + self.len() - 1)
            .unwrap()
    }
}

impl Debug for SourceView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Span")
            .field("arena", &"omitted")
            .field("start", &self.start())
            .field("length", &self.len())
            .finish()
    }
}

impl Display for SourceView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let slice = self.clone().into_string();
        f.write_str(&*slice);
        write!(f, "{}", slice)
    }
}

impl Add for SourceView {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        assert_pre_condition!(self.start() <= rhs.start() && self.end() <= rhs.end());
        let new_start = self.span.offset();
        let new_len = rhs.len() - new_start;
        Self {
            arena: self.arena,
            span: SourceSpan::from((new_start, new_len)),
        }
    }
}
