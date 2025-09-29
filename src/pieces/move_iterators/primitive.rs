use std::ops::Range;

use super::*;

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

pub struct LMoveIter {
    dimensions: isize,
    steps: Vec<Range<isize>>,
    next: Vec<isize>,
    done: bool,
}

impl LMoveIter {
    pub fn new(dimensions: usize, steps: usize) -> LMoveIter {
        debug_assert!(steps < 8);
        let mut vec = Vec::new();
        let dimensions = dimensions as isize;

        for _ in 0..steps {
            vec.push(-dimensions..dimensions);
        }

        LMoveIter {
            dimensions,
            steps: vec,
            next: vec![1; steps],
            done: false,
        }
    }

    fn update_next(&mut self) {
        let len = self.steps.len() - 1;
        for (i, step) in self.steps.iter_mut().enumerate() {
            let mut next = step.next();
            if let Some(0) = next {
                next = step.next();
            };

            if let Some(next) = next {
                self.next[i] = next;
                break;
            } else {
                *step = (-self.dimensions + 1)..self.dimensions + 1;
                self.next[i] = -self.dimensions;
                if i == len {
                    self.done = true;
                }
            }
        }
    }

    fn check_valid(&self) -> bool {
        for i in 0..self.next.len() {
            for j in (i + 1)..self.next.len() {
                if self.next[i].abs() == self.next[j].abs() {
                    return false;
                }
            }
        }
        true
    }
}

impl Iterator for LMoveIter {
    type Item = Position;
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        for _ in 0..1000000 {
            self.update_next();
            if self.check_valid() {
                break;
            }
        }
        if self.done {
            return None;
        }

        let mut moves = Position(vec![0; self.dimensions as usize]);

        for (by, &dim) in self.next.iter().enumerate() {
            let pos = dim.signum() as i8;
            let dimension = dim.unsigned_abs() - 1;
            moves.add_dimension(dimension, pos * (by as i8 + 1));
        }
        Some(moves)
    }
}
