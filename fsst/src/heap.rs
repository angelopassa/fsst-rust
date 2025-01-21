use std::cmp::Ordering;

use crate::symbol::Symbol;

#[derive(Eq, PartialEq)]
pub struct HeapPair(pub usize, pub Symbol);

impl Ord for HeapPair {
    fn cmp(&self, other: &Self) -> Ordering {
        let ord = self.0.cmp(&other.0);
        match ord {
            Ordering::Equal => self.1.len.cmp(&other.1.len),
            _ => ord,
        }
    }
}

impl PartialOrd for HeapPair {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
