use std::fmt::Display;

use append_only_vec::AppendOnlyVec;

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
    fn to_string(&self) -> String {
        self.arena
            .iter()
            .skip(self.start)
            .take(self.length)
            .collect()
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let slice = self.to_string();
        write!(f, "{}", slice)
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

pub struct Reader {
    pub current: Span,
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
        }
    }

    pub fn is_at_end(&self) -> bool {
        self.current.start >= self.current.arena.len()
    }

    /// look ahead in iterator without advancing
    pub fn peek(&self) -> Option<char> {
        if self.is_at_end() {
            None
        } else {
            Some(self.current.arena[self.current.start])
        }
    }

    /// look ahead 2 chars without advancing
    pub fn peek2(&self) -> Option<char> {
        if self.current.start + 1 >= self.current.arena.len() {
            None
        } else {
            Some(self.current.arena[self.current.start + 1])
        }
    }
}

impl Iterator for Reader {
    type Item = char;

    /// get next char, ignores the length of the window and leaves it unchanged
    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.peek()?;
        self.current.start += 1;
        Some(ret)
    }
}

// /// Get singleton writer to underlying arena
// fn arena_writer() -> RwLockWriteGuard<'static, Vec<String>> {
//     STRING_ARENA
//         .get_or_init(|| RwLock::new(vec![]))
//         .write()
//         .unwrap_or_else(|_| fatal_generic("poisoned lock"))
// }

// /// Get singleton reader to underlying arena, prefer using this over
// /// `arena_writer` as it is more efficient
// fn arena_reader() -> RwLockReadGuard<'static, Vec<String>> {
//     STRING_ARENA
//         .get_or_init(|| RwLock::new(vec![]))
//         .read()
//         .unwrap_or_else(|_| fatal_generic("poisoned lock"))
// }
