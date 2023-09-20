use std::{
    fmt::{Debug, Display},
    ops::{Add, AddAssign},
};

use append_only_vec::AppendOnlyVec;

use crate::assert_pre_condition;

// TODO: move into struct when you are ready to deal with the lifetime
//       mess... ugh...
static STRING_ARENA: AppendOnlyVec<char> = AppendOnlyVec::new();

#[derive(Clone, Copy)]
pub struct Span {
    arena: &'static AppendOnlyVec<char>,
    pub start: usize,
    pub length: usize,
}

impl Span {
    pub fn into_string(self) -> String {
        self.arena
            .iter()
            .skip(self.start)
            .take(self.length)
            .collect()
    }

    fn end(&self) -> usize {
        self.start + self.length
    }

    /// get a char from the span given index
    pub fn get(&self, i: usize) -> char {
        // pre-condition: index in range
        crate::assert_pre_condition!(i < self.length);

        self.arena[i - self.start]
    }

    /// get a char from the span indexing from the end
    pub fn get_back(&self, i: usize) -> char {
        // pre-condition: index in range
        crate::assert_pre_condition!(self.start + self.length - i < self.length);

        self.arena[self.start + self.length - 1]
    }
}

impl Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Span")
            .field("arena", &"omitted")
            .field("start", &self.start)
            .field("length", &self.length)
            .finish()
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let slice = self.into_string();
        write!(f, "{:?}", slice)
    }
}

impl Add for Span {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        assert_pre_condition!(self.start <= rhs.start && self.end() <= rhs.end());
        Self {
            arena: self.arena,
            start: self.start,
            length: rhs.end() - self.start,
        }
    }
}

/// Intern a single string containing multiple lines of unprcessed source
pub fn intern(src: String) -> Span {
    let start = STRING_ARENA.len();
    let length = src.len();

    src.chars().for_each(|c| {
        STRING_ARENA.push(c);
    });

    Span {
        arena: &STRING_ARENA,
        start,
        length,
    }
}

/// Dump contents of arena
pub fn dump() -> String {
    STRING_ARENA.iter().collect()
}

pub struct Reader {
    pub current: Span,
    boundary: ReaderBounds,
}

enum ReaderBounds {
    Absolute,
    Relative(Span),
}

/// fast read-only iterator over arena
///
/// reader can be advanced one character at a time simply by using it as
/// an iterator, or the start and end can be advanced separately for
/// lexing.
impl Reader {
    /// crate a new reader that traverses the entire arena from the start
    pub fn new() -> Self {
        Self {
            current: Span {
                arena: &STRING_ARENA,
                start: 0,
                length: 0,
            },
            boundary: ReaderBounds::Absolute,
        }
    }

    pub fn from_span(s: Span) -> Self {
        Self {
            current: Span {
                arena: s.arena,
                start: 0,
                length: 0,
            },
            boundary: ReaderBounds::Relative(s),
        }
    }

    pub fn abs_bounds(&self) -> (usize, usize) {
        match self.boundary {
            ReaderBounds::Absolute => (0, self.current.arena.len()),
            ReaderBounds::Relative(s) => (s.start, s.start + s.length),
        }
    }

    pub fn rel_bounds(&self) -> usize {
        match self.boundary {
            ReaderBounds::Absolute => self.current.arena.len(),
            ReaderBounds::Relative(s) => s.length,
        }
    }

    pub fn is_at_end(&self) -> bool {
        self.head_idx() >= self.rel_bounds()
    }

    /// look ahead in iterator without advancing
    pub fn peek(&self) -> Option<char> {
        if self.is_at_end() {
            None
        } else {
            Some(self.current.arena[self.head_idx()])
        }
    }

    pub fn peek_n(&self, n: usize) -> Option<char> {
        if self.head_idx() + n >= self.rel_bounds() {
            None
        } else {
            Some(self.current.arena[self.head_idx() + n])
        }
    }

    pub fn advance_head(&mut self) -> Option<char> {
        let ret = self.peek()?;
        self.current.length += 1;
        Some(ret)
    }

    pub fn advance_tail(&mut self) -> Span {
        let ret = self.current;
        self.current.start = self.head_idx();
        self.current.length = 0;
        ret
    }

    /// get the index of the current char
    fn head_idx(&self) -> usize {
        self.current.start + self.current.length
    }
}

impl Iterator for Reader {
    type Item = char;

    /// get next char, ignores the length of the window and leaves it unchanged
    fn next(&mut self) -> Option<Self::Item> {
        self.advance_head()
    }
}
