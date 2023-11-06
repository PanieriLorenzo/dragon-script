use std::rc;

use super::*;

#[derive(Clone)]
pub struct SourceString {
    pub(crate) source: rc::Weak<Source>,
    pub(crate) pos: Range<usize>,
}

#[derive(Clone)]
pub enum Production {
    /// A raw string segment directly from the source
    Atom(SourceString),

    /// A production made of several disjoint segments
    Fused(Vec<SourceString>)
}



// impl SourceString {
//     pub fn from_source(source: &Rc<Source>) -> Self {
//         Self {
//             source: Rc::downgrade(source),
//             pos: 0..arena.len(),
//         }
//     }

//     pub fn grow(&mut self) {
//         self.span = self.span.start..(self.span.end + 1); //SourceSpan::from((self.span.offset(), self.span.len() + 1));
//     }

//     /// Increment offset up to current end, and reset length to 0
//     pub fn pull_tail(&mut self) {
//         self.span = self.span.end..self.span.end; //SourceSpan::from((self.end(), 0));
//     }

//     pub fn start(&self) -> usize {
//         self.span.start
//     }

//     pub fn len(&self) -> usize {
//         self.span.end - self.span.start
//     }

//     pub fn end(&self) -> usize {
//         self.span.end
//     }

//     pub fn into_string(self) -> String {
//         self.arena
//             .upgrade()
//             .unwrap()
//             .inner()
//             .iter()
//             .skip(self.start())
//             .take(self.len())
//             .collect()
//     }

//     /// get a char from the span given index
//     pub fn get(&self, i: usize) -> char {
//         // pre-condition: index in range
//         crate::assert_pre_condition!(i < self.len());

//         self.arena.upgrade().unwrap().get(i - self.len()).unwrap()
//     }

//     /// get a char from the span indexing from the end
//     pub fn get_back(&self, i: usize) -> char {
//         // pre-condition: index in range
//         crate::assert_pre_condition!(self.start() + self.len() - i < self.len());

//         self.arena
//             .upgrade()
//             .unwrap()
//             .get(self.start() + self.len() - 1)
//             .unwrap()
//     }
// }

// impl Debug for SourceView {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("Span")
//             .field("arena", &"omitted")
//             .field("start", &self.start())
//             .field("length", &self.len())
//             .finish()
//     }
// }

// impl Display for SourceView {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let slice = self.clone().into_string();
//         write!(f, "{}", slice)
//     }
// }

// impl Add for SourceView {
//     type Output = Self;

//     fn add(self, rhs: Self) -> Self::Output {
//         assert_pre_condition!(self.start() <= rhs.start() && self.end() <= rhs.end());
//         Self {
//             arena: self.arena,
//             span: self.span.start..rhs.span.end,
//             source_id: todo!(),
//         }
//     }
// }

// impl From<SourceView> for Range<usize> {
//     fn from(val: SourceView) -> Self {
//         (val.span.start + 1)..(val.span.end + 1)
//     }
// }
