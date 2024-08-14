use rand::{distributions::Standard, prelude::Distribution, Rng};

use super::{color::TetriminoColor, piece::Piece, Offset};

// derive traits
#[derive(Clone, Copy, Debug, PartialEq)]

// all of the types of pieces (by shape)
pub enum PieceKind {
    O,
    I,
    T,
    L,
    J,
    S,
    Z,
}

impl PieceKind {
    // static array of all the different variants
    pub const ALL: [Self; 7] = [
        Self::O,
        Self::I,
        Self::T,
        Self::L,
        Self::J,
        Self::S,
        Self::Z,
    ];

    // get piece "coordinates" - north facing representations
    // coordinates are starting from always the same origin point of the tetrimino!
    pub fn cells(&self, // shared reference to self
    ) -> [Offset; Piece::CELL_COUNT] {
        match self {
            Self::O => &[(1, 1), (1, 2), (2, 1), (2, 2)],
            Self::I => &[(0, 2), (1, 2), (2, 2), (3, 2)],
            Self::T => &[(0, 1), (1, 1), (2, 1), (1, 2)],
            Self::L => &[(0, 1), (1, 1), (2, 1), (2, 2)],
            Self::J => &[(0, 2), (0, 1), (1, 1), (2, 1)],
            Self::S => &[(0, 1), (1, 1), (1, 2), (2, 2)],
            Self::Z => &[(0, 2), (1, 2), (1, 1), (2, 1)],
        }
        .map(Offset::from) // map to vector
    }

    pub fn grid_size(&self) -> isize {
        match self {
            PieceKind::I => 4,
            _ => 3,
        }
    }

    pub fn color(&self) -> TetriminoColor {
        match self {
            Self::O => TetriminoColor::Yellow,
            Self::I => TetriminoColor::Cyan,
            Self::T => TetriminoColor::Purple,
            Self::L => TetriminoColor::Orange,
            Self::J => TetriminoColor::Blue,
            Self::S => TetriminoColor::Green,
            Self::Z => TetriminoColor::Red,
        }
    }
}

// this is so we can assign a random tetrimino
impl Distribution<PieceKind> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> PieceKind {
        match rng.gen_range(0..7) {
            0 => PieceKind::I,
            1 => PieceKind::J,
            2 => PieceKind::L,
            3 => PieceKind::O,
            4 => PieceKind::S,
            5 => PieceKind::T,
            6 => PieceKind::Z,
            _ => PieceKind::T, // default to T
        }
    }
}
