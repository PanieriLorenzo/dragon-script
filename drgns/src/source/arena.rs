use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    fmt::Display,
    rc::Rc,
    sync::{RwLock, RwLockReadGuard},
};

use bimap::BiMap;

use super::view::SourceView;

#[derive(Debug)]
#[deprecated]
pub struct SourceArena {
    data: RwLock<Vec<char>>,
    ids: RefCell<BiMap<String, u16>>,
}

impl Default for SourceArena {
    fn default() -> Self {
        Self::new()
    }
}

impl SourceArena {
    pub fn new() -> Self {
        let mut ret = Self {
            data: RwLock::new(vec![]),
            ids: RefCell::new(BiMap::new()),
        };
        ret.ids.borrow_mut().insert("REPL".to_string(), 0);
        ret
    }

    /// Intern a single string of raw source code, including newlines.
    ///
    /// You may intern parts of a single line, or multiple lines as well.
    pub fn intern(self: &Rc<Self>, src: String) -> SourceView {
        log::trace!("interning string: '{:?}'", src);
        let start = self.len();
        self.data.write().unwrap().extend(src.chars());
        SourceView {
            arena: Rc::<SourceArena>::downgrade(self),
            span: start..(start + src.len()),
            source_id: todo!(),
        }
    }

    pub fn len(&self) -> usize {
        self.data.read().unwrap().len()
    }

    pub fn get(&self, idx: usize) -> Option<char> {
        self.data.read().unwrap().get(idx).copied()
    }

    pub fn inner(&self) -> RwLockReadGuard<'_, Vec<char>> {
        self.data.read().unwrap()
    }
}

impl Display for SourceArena {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String = self.inner().iter().collect();
        write!(f, "{}", s)
    }
}


#[derive(Clone)]
#[deprecated]
pub struct Reader {
    pub current: SourceView,
    boundary: ReaderBounds,
}

#[deprecated]
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
    pub fn from_arena(s: &Rc<SourceArena>) -> Self {
        Self {
            current: SourceView {
                arena: Rc::downgrade(s),
                span: 0..0,
                source_id: todo!(),
            },
            boundary: ReaderBounds::Absolute,
        }
    }

    pub fn from_span(s: SourceView) -> Self {
        Self {
            current: SourceView {
                arena: s.arena.clone(),
                span: 0..0,
                source_id: todo!(),
            },
            boundary: ReaderBounds::Relative(s),
        }
    }

    pub fn abs_bounds(&self) -> (usize, usize) {
        match &self.boundary {
            ReaderBounds::Absolute => (0, self.current.arena.upgrade().unwrap().len()),
            ReaderBounds::Relative(s) => (s.span.start, s.span.end),
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
