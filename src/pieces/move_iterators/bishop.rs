use super::*;

pub struct BishopMoveIterator {
    move_set: u64,
    len: usize,
}

impl Iterator for BishopMoveIterator {
    type Item = DiagonalIter<7>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.move_set >= 1 << self.len {
            return None;
        }
        let out = DiagonalIter::with_move_set(self.gen_move_set());
        self.move_set += 1;
        Some(out)
    }
}

impl BishopMoveIterator {
    pub fn new(dimensions: usize) -> BishopMoveIterator {
        BishopMoveIterator {
            move_set: 0,
            len: dimensions,
        }
    }

    fn gen_move_set(&self) -> Vec<bool> {
        let mut out = vec![false; self.len];
        for i in 0..self.len {
            out[i] = (self.move_set & (1 << i)) != 0;
        }
        out
    }
}
