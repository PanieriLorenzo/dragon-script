use std::{
    convert::identity,
    fmt::Display,
    sync::{Arc, RwLock, RwLockReadGuard},
};

use miette::{LabeledSpan, SourceOffset, SourceSpan};

use super::view::SourceView;

#[derive(Debug)]
pub struct SourceArena(RwLock<Vec<char>>);

impl SourceArena {
    pub fn new() -> Self {
        Self(RwLock::new(vec![]))
    }

    /// Intern a single string of raw source code, including newlines.
    ///
    /// You may intern parts of a single line, or multiple lines as well.
    pub fn intern(self: &Arc<Self>, src: String) -> SourceView {
        let start: SourceOffset = self.len().into();
        self.0.write().unwrap().extend(src.chars());
        SourceView {
            arena: Arc::<SourceArena>::downgrade(self),
            span: SourceSpan::new(start, src.len().into()),
        }
    }

    pub fn len(&self) -> usize {
        self.0.read().unwrap().len()
    }

    pub fn get(&self, idx: usize) -> Option<char> {
        self.0.read().unwrap().get(idx).map(|&c| c)
    }

    pub fn inner(&self) -> RwLockReadGuard<'_, Vec<char>> {
        self.0.read().unwrap()
    }
}

#[derive(Clone)]
pub struct Reader {
    pub current: SourceView,
    boundary: ReaderBounds,
}

#[derive(Clone)]
enum ReaderBounds {
    Absolute,
    Relative(SourceView),
}

/// fast read-only iterator over arena
///
/// reader can be advanced one character at a time simply by using it as
/// an iterator, or the start and end can be advanced separately for
/// lexing.
impl Reader {
    /// crate a new reader that traverses the entire arena from the start
    pub fn from_arena(s: &Arc<SourceArena>) -> Self {
        Self {
            current: SourceView {
                arena: Arc::downgrade(&s),
                span: SourceSpan::new(0.into(), 0.into()),
            },
            boundary: ReaderBounds::Absolute,
        }
    }

    pub fn from_span(s: SourceView) -> Self {
        Self {
            current: SourceView {
                arena: s.arena.clone(),
                span: SourceSpan::new(0.into(), 0.into()),
            },
            boundary: ReaderBounds::Relative(s),
        }
    }

    pub fn abs_bounds(&self) -> (usize, usize) {
        match &self.boundary {
            ReaderBounds::Absolute => (0, self.current.arena.upgrade().unwrap().len()),
            ReaderBounds::Relative(s) => (s.span.offset(), s.span.offset() + s.span.len()),
        }
    }

    pub fn rel_bounds(&self) -> usize {
        match &self.boundary {
            ReaderBounds::Absolute => self.current.arena.upgrade().unwrap().len(),
            ReaderBounds::Relative(s) => s.span.len(),
        }
    }

    pub fn is_at_end(&self) -> bool {
        self.current.end() >= self.rel_bounds()
    }

    /// look ahead in iterator without advancing
    pub fn peek(&self) -> Option<char> {
        if self.is_at_end() {
            None
        } else {
            Some(
                self.current
                    .arena
                    .upgrade()
                    .unwrap()
                    .get(self.current.end())
                    .unwrap(),
            )
        }
    }

    pub fn peek_n(&self, n: usize) -> Option<char> {
        if self.current.end() + n >= self.rel_bounds() {
            None
        } else {
            Some(
                self.current
                    .arena
                    .upgrade()
                    .unwrap()
                    .get(self.current.end() + n)
                    .unwrap(),
            )
        }
    }

    pub fn advance_head(&mut self) -> Option<char> {
        let ret = self.peek()?;
        self.current.grow();
        Some(ret)
    }

    pub fn advance_tail(&mut self) -> SourceView {
        let ret = self.current.clone();
        self.current.pull_tail();
        ret
    }
}

impl Iterator for Reader {
    type Item = char;

    /// get next char, ignores the length of the window and leaves it unchanged
    fn next(&mut self) -> Option<Self::Item> {
        self.advance_head()
    }
}
