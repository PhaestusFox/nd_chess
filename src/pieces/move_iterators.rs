use crate::board::Position;

mod bishop;
mod knight;
mod primitive;

pub use bishop::BishopMoveIterator;
pub use knight::KnightMoveIterator;
pub use primitive::DiagonalIter;
pub use primitive::LMoveIter;
