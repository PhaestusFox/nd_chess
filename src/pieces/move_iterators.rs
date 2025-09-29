use crate::board::Position;

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

pub struct DiagonalIter<const DISTANCE: u8> {
    current: Position,
    move_set: Vec<bool>,
    step: u8,
}

impl<const DISTANCE: u8> DiagonalIter<DISTANCE> {
    pub fn new(dimensions: usize) -> Self {
        DiagonalIter {
            current: Position(vec![0; dimensions]),
            move_set: vec![false; dimensions],
            step: 0,
        }
    }
    pub fn with_move_set(set: Vec<bool>) -> DiagonalIter<DISTANCE> {
        DiagonalIter {
            current: Position(vec![0; set.len()]),
            move_set: set,
            step: 0,
        }
    }
}

impl<const DISTANCE: u8> Iterator for DiagonalIter<DISTANCE> {
    type Item = Position;
    fn next(&mut self) -> Option<Self::Item> {
        if self.step > DISTANCE {
            return None;
        }
        self.step += 1;
        let out = self.current.clone();
        for (dim, &dir) in self.move_set.iter().enumerate() {
            if dir {
                self.current.inc_in_place(dim);
            } else {
                self.current.dec_in_place(dim);
            }
        }
        Some(out)
    }
}
