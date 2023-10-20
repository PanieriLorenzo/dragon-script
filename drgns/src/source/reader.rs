use std::rc::Weak;

use crate::lookahead::Cursor;

use super::*;

#[derive(Clone)]
pub struct Reader {
    pub source: Weak<Source>,
    back: usize,
    front: usize,
}

impl Reader {
    fn inner_len(&self) -> Option<usize> {
        Some(self.source.upgrade()?.src.len())
    }

    fn exhausted(&self) -> bool {
        if self.inner_len().is_none() {
            return true;
        }

        let len = self.inner_len().expect("infallible");
        self.front >= len
    }

    fn get_from_inner(&self, i: usize) -> Option<char> {
        self.source.upgrade()?.src.get(i).copied()
    }
}

impl Cursor<char, SourceString> for Reader {
    fn peek_n(&mut self, i: usize) -> Option<char> {
       self.get_from_inner(self.front + i)
    }

    fn previous(&self) -> Option<char> {
        self.get_from_inner(self.front - 1)
    }

    fn peek_back_n(&self, i: usize) -> Option<char> {
        self.get_from_inner(self.back + i)
    }

    fn window_len(&self) -> usize {
        self.front - self.back
    }

    fn window_is_empty(&self) -> bool {
        self.front - self.back == 0
    }

    fn advance(&mut self) -> Option<char> {
        // early return if exhausted to avoid incrementing unnecessarily
        if self.exhausted() {
            return None;
        }

        let ret = self.get_from_inner(self.front);
        self.front += 1;
        ret
    }

    fn consume(&mut self) -> Option<SourceString> {
        let ret = Some(SourceString{source: self.source.clone(), pos: self.back..self.front});

        self.back = self.front;

        if self.exhausted() {
            return None;
        }

        ret
    }

    fn reset(&mut self) {
        self.front = self.back;
    }
}
