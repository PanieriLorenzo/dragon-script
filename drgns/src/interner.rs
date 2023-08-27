use std::fmt::Display;

use append_only_vec::AppendOnlyVec;

//static STRING_ARENA: OnceLock<RwLock<Vec<String>>> = OnceLock::new();
static STRING_ARENA: AppendOnlyVec<String> = AppendOnlyVec::new();

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub lin_start: usize,
    pub col_start: usize,
    pub lin_len: usize,
    pub col_len: usize,
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut slice = STRING_ARENA
            .iter()
            .skip(self.lin_start)
            .take(self.lin_len)
            .cloned()
            .collect::<Vec<_>>();

        let last_lin_idx = slice.len() - 1;

        if last_lin_idx == 0 {
            // if start and end are on the same line, we need to slice from
            // col_start to col_end on that one line
            slice[0] = slice[0][self.col_start..(self.col_start + self.col_len)].to_owned();
        } else {
            // if start and end are on different lines, we can slice them
            // independently from each other
            slice[0] = slice[0][self.col_start..].to_owned();
            slice[last_lin_idx] = slice[last_lin_idx][..self.col_len].to_owned();
        }

        write!(f, "{}", slice.join("\n"))
    }
}

/// Intern a single string containing multiple lines of unprcessed source
pub fn intern_raw(src: String) -> Vec<Span> {
    src.lines()
        .map(|lin| {
            let lin_idx = STRING_ARENA.push(lin.to_owned());
            Span {
                lin_start: lin_idx,
                col_start: 0,
                lin_len: 1,
                col_len: lin.len(),
            }
        })
        .collect()
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
