//! An iterator that keeps a lookahead until it is committed, similar to Multipeek

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
