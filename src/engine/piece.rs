use cgmath::Zero;
use rand::distributions::Standard;
use rand::prelude::Distribution;
use rand::Rng;

use super::{Color, Matrix};
use super::{Coordinate, Offset};
use cgmath::EuclideanSpace;

#[derive(Clone, Copy, PartialEq, Debug)]

pub struct Piece {
    pub kind: Kind,       // these do not need to be in a constructor
    pub position: Offset, // holds x & y
    pub rotation: Rotation,
}

impl Piece {
    pub const CELL_COUNT: usize = 4;

    // will return a new piece in the new moved position
    pub fn moved_by(&self, offset: Offset) -> Self {
        Self {
            position: self.position + offset,
            ..*self
        }
    }

    // returns coordinates of piece; None on an invalid cursor position;
    // returns an array of length CELL_COUNT
    pub fn cells(&self) -> Option<[Coordinate; Self::CELL_COUNT]> {
        // array of 4 offsets which we need to convert into coordinates
        let offsets = self.kind.cells().map(self.rotator()).map(self.positioner());

        let mut coords = [Coordinate::origin(); Self::CELL_COUNT];

        // convert to coords
        for (offset, coord_slot) in offsets.into_iter().zip(&mut coords) {
            // cast to a positive integer and let it throw if it can't be
            let positive_offset = offset.cast::<usize>()?; // the question mark denotes that if this returns none, the whole thing will return none
            let coord = Coordinate::from_vec(positive_offset);

            // check that the position is within bounds, the negative check is already done by the conversion above
            if Matrix::valid_coord(coord) {
                *coord_slot = coord;
            } else {
                return None;
            }
        }

        Some(coords)
    }

    // will rotate a single tetrimino
    fn rotator(&self) -> impl Fn(Offset) -> Offset + '_ {
        // to capture lifetime of self, add '_
        |cell| match self.kind {
            Kind::O => cell, // skip rotation for square as it's defined as 3x3 and it's not rotated
            _ => {
                let rotated = cell * self.rotation;
                // add in the intrinsic offset multiplied by the grid size
                let grid_offset = self.rotation.intrinsic_offset() * (self.kind.grid_size() - 1); // 0 is shared with these rotations so we need to move by grid size -1!
                rotated + grid_offset
            }
        }
    }

    // will move the cell into position
    fn positioner(&self) -> impl Fn(Offset) -> Offset {
        let position: Offset = self.position;
        move |cell: Offset| cell + position // move it into address space
    }
}

// derive traits
#[derive(Clone, Copy, Debug, PartialEq)]

// all of the types of pieces (by shape)
pub enum Kind {
    O,
    I,
    T,
    L,
    J,
    S,
    Z,
}

impl Kind {
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
    fn cells(&self, // shared reference to self
    ) -> [Offset; Piece::CELL_COUNT] {
        match self {
            Self::O => &[(1, 1), (1, 2), (2, 1), (1, 1)],
            Self::I => &[(0, 2), (1, 2), (2, 2), (2, 2)],
            Self::T => &[(0, 1), (1, 1), (2, 1), (1, 2)],
            Self::L => &[(0, 1), (1, 1), (2, 1), (2, 2)],
            Self::J => &[(0, 2), (0, 1), (1, 1), (2, 1)],
            Self::S => &[(0, 1), (1, 1), (1, 2), (2, 2)],
            Self::Z => &[(0, 2), (1, 2), (1, 1), (2, 1)],
        }
        .map(Offset::from) // map to vector
    }

    fn grid_size(&self) -> isize {
        match self {
            Kind::I => 4,
            _ => 3,
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Self::O => Color::Yellow,
            Self::I => Color::Cyan,
            Self::T => Color::Purple,
            Self::L => Color::Orange,
            Self::J => Color::Blue,
            Self::S => Color::Green,
            Self::Z => Color::Blue,
        }
    }
}

impl Distribution<Kind> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Kind {
        match rng.gen_range(0..7) {
            0 => Kind::I,
            1 => Kind::J,
            2 => Kind::L,
            3 => Kind::O,
            4 => Kind::S,
            5 => Kind::T,
            6 => Kind::Z,
            _ => Kind::T, // default to T
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
// how the piece is rotated
pub enum Rotation {
    N,
    E,
    S,
    W,
}

impl Rotation {
    fn intrinsic_offset(&self) -> Offset {
        // this we need to then multiply by grid size
        match self {
            Self::N => Offset::zero(),
            Self::E => Offset::new(0, 1), // 2nd quadrant, so y has moved
            Self::S => Offset::new(1, 1), // 3rd quadrant, so both x and y have moved down
            Self::W => Offset::new(1, 0), // 4th quadrant, so only x has moved
        }
    }

    pub fn next_rotation(&self) -> Self {
        match self {
            Self::N => Self::E,
            Self::E => Self::S,
            Self::S => Self::W,
            Self::W => Self::N,
        }
    }
}

// multiply vector by a rotation -> for rotating relative coordinates of a piece
impl std::ops::Mul<Rotation> for Offset {
    type Output = Self;

    fn mul(self, rotation: Rotation) -> Self::Output {
        match rotation {
            Rotation::N => self, // no op as the coordinates are already north facing
            Rotation::S => Self::new(-self.x, -self.y), // flip x & y axis
            Rotation::E => Self::new(self.y, -self.x),
            Rotation::W => Self::new(-self.y, self.x),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn s_piece_positioning() {
        let z = Piece {
            kind: Kind::Z,
            position: Offset::new(5, 6),
            rotation: Rotation::W,
        };
        assert_eq!(
            z.cells(),
            Some([(5, 6), (5, 7), (6, 7), (6, 8)].map(Coordinate::from))
        );
    }
}
