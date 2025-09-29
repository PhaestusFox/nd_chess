use super::*;

pub struct KnightMoveIterator {
    dimensions: usize,
    next: usize,
}

impl KnightMoveIterator {
    pub fn new(dimensions: usize) -> KnightMoveIterator {
        KnightMoveIterator {
            dimensions,
            next: 1,
        }
    }
}

impl Iterator for KnightMoveIterator {
    type Item = LMoveIter;
    fn next(&mut self) -> Option<Self::Item> {
        if self.next > self.dimensions {
            return None;
        }
        self.next += 1;
        Some(LMoveIter::new(self.dimensions, self.next))
    }
}
