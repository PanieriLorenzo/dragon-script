//! An iterator that keeps a lookahead until it is committed, similar to Multipeek

#[deprecated]
pub struct Lookahead<I: Iterator + Clone>
where
    I::Item: Clone,
{
    main: I,
    branch: I,
    pub current: Option<I::Item>,
}

impl<I: Iterator + Clone> Lookahead<I>
where
    <I as Iterator>::Item: Clone,
{
    pub fn peek(&mut self) -> Option<I::Item> {
        self.current = self.branch.next();
        self.current.clone()
    }

    pub fn reset(&mut self) {
        self.branch = self.main.clone()
    }

    pub fn commit(&mut self) {
        self.main = self.branch.clone()
    }
}

impl<I: Iterator + Clone> Iterator for Lookahead<I>
where
    <I as Iterator>::Item: Clone,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.current = self.main.next();
        self.branch = self.main.clone();
        self.current.clone()
    }
}

pub fn lookahead<I: IntoIterator>(iterable: I) -> Lookahead<I::IntoIter>
where
    <I as IntoIterator>::IntoIter: Clone,
    <I as IntoIterator>::Item: Clone,
{
    let main = iterable.into_iter();
    let branch = main.clone();
    let current = None;
    Lookahead {
        main,
        branch,
        current,
    }
}

/// Complex and highly flexible iterator that allows arbitrary lookahead, 1-value of
/// lookbehind and building spans of values
pub trait Cursor<T, S> {
    /// Look ahead n-positions without advancing front or back indexes, index
    /// is relative to the position of the front index, so n = 0 returns the
    /// same value that [`advance`] would return.
    fn peek_n(&mut self, i: usize) -> Option<T>;

    /// Peek at the last emitted value from [`advance`], conceptually the same as peek_n(-1)
    fn previous(&self) -> Option<T>;

    /// Peek at the next value emitted from [`advance`], without staging it
    fn front(&mut self) -> Option<T> {
        self.peek_n(0)
    }

    /// Peek at the value pointed at by the back iterator
    fn back(&self) -> Option<T> {
        self.peek_back_n(0)
    }

    /// Peek n values from the back iterator, note that this may not cross into
    /// the lookahead range.
    fn peek_back_n(&self, i: usize) -> Option<T>;

    /// Get the length of the span between the back and front iterators
    fn window_len(&self) -> usize;

    /// Check if window is empty, e.g. if [`reset`] was called.
    fn window_is_empty(&self) -> bool;

    /// Advance the front iterator, yielding a value
    fn advance(&mut self) -> Option<T>;

    /// Advance the back iterator all the way to the front iterator, yielding a span
    fn consume(&mut self) -> Option<S>;

    /// Advance the back iterator all the way to the front iterator, discarding the
    /// value
    fn burn(&mut self) {
        self.consume();
    }

    /// Backtrack the front iterator to the back iterator
    fn reset(&mut self);
}

#[cfg(test)]
mod test {
    use std::collections::VecDeque;

    use super::Cursor;

    struct VecCursor {
        v: Vec<i32>,
        back: usize,
        front: usize,
        previous: Option<i32>,
    }

    impl VecCursor {
        pub fn new(v: Vec<i32>) -> Self {
            Self {
                v,
                back: 0,
                front: 0,
                previous: None,
            }
        }
    }

    impl Cursor<i32, Vec<i32>> for VecCursor {
        fn peek_n(&mut self, i: usize) -> Option<i32> {
            self.v.get(self.front + i).copied()
        }

        fn previous(&self) -> Option<i32> {
            self.previous
        }

        fn peek_back_n(&self, i: usize) -> Option<i32> {
            self.v.get(self.back + i).copied()
        }

        fn window_len(&self) -> usize {
            self.front - self.back
        }

        fn window_is_empty(&self) -> bool {
            self.window_len() == 0
        }

        fn advance(&mut self) -> Option<i32> {
            let ret = self.v.get(self.front);
            self.front += 1;
            self.previous = ret.copied();
            ret.copied()
        }

        fn consume(&mut self) -> Option<Vec<i32>> {
            if self.back >= self.v.len() || self.front >= self.v.len() {
                return None;
            }
            let ret = self.v[self.back..self.front].to_vec();
            self.back = self.front;
            Some(ret)
        }

        fn reset(&mut self) {
            self.front = self.back;
        }
    }

    #[test]
    fn cursor() {
        let mut c = VecCursor::new(vec![1, 2, 3, 4, 5, 6]);
        assert!(c.window_is_empty());
        assert_eq!(c.advance(), Some(1));
        assert_eq!(c.advance(), Some(2));
        assert_eq!(c.previous(), Some(2));
        assert_eq!(c.back(), Some(1));
        assert_eq!(c.consume(), Some(vec![1, 2]));
        assert!(c.window_is_empty());
        assert_eq!(c.advance(), Some(3));
        assert_eq!(c.window_len(), 1);
        c.reset();
        assert_eq!(c.peek_n(2), Some(5));
        assert_eq!(c.advance(), Some(3));
        assert_eq!(c.advance(), Some(4));
        assert_eq!(c.advance(), Some(5));
        assert_eq!(c.peek_back_n(1), Some(4));
        assert_eq!(c.front(), Some(6));
        assert_eq!(c.advance(), Some(6));
    }
}
