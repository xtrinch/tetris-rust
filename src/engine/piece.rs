use cgmath::Vector2;
use cgmath::Zero;

use super::Matrix;
use super::{Coordinate, Offset};

pub(super) struct Piece {
    pub kind: Kind,       // these do not need to be in a constructor
    pub position: Offset, // holds x & y
    pub rotation: Rotation,
}

impl Piece {
    const CELL_COUNT: usize = 4;

    // returns coordinates of piece
    pub fn cells(&self) -> Option<[Coordinate; Self::CELL_COUNT]> {
        // array of 4 offsets which we need to convert into coordinates
        let offsets = self.kind.cells().map(self.rotator()).map(self.positioner());

        let mut coords = [Coordinate::zero(); Self::CELL_COUNT];

        // convert to coords
        for (Offset { x, y }, coord) in offsets.into_iter().zip(&mut coords) {
            // convert to an unsigned number
            let new = match (x.try_into(), y.try_into()) {
                (Ok(x), Ok(y)) => Coordinate { x, y },
                _ => return None,
            };

            // check that the position is within bounds, the negative check is already done by the conversion above
            if Matrix::in_bounds(new) {
                *coord = new;
            } else {
                return None;
            }
        }

        Some(coords)
    }

    // will rotate a single cell
    fn rotator(&self) -> impl Fn(Offset) -> Offset {
        let rotation: Rotation = self.rotation;
        move |cell: Offset| cell * rotation // move it into address space
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
    // coordinates are from their center of rotation
    fn cells(&self, // shared reference to self
    ) -> [Offset; Piece::CELL_COUNT] {
        match self {
            Kind::O => &[(0, 0), (0, 1), (1, 0), (1, 1)],
            Kind::I => &[(-1, 0), (0, 0), (1, 0), (2, 0)],
            Kind::T => &[(-1, 0), (0, 0), (1, 0), (0, 1)],
            Kind::L => &[(-1, 0), (0, 0), (1, 0), (1, 1)],
            Kind::J => &[(-1, 1), (-1, 0), (0, 0), (1, 0)],
            Kind::S => &[(-1, 0), (0, 0), (0, 1), (1, 1)],
            Kind::Z => &[(-1, 1), (0, 1), (0, 0), (1, 0)],
        }
        .map(Offset::from) // map to vector
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

// multiply vector by a rotation -> for rotating relative coordinates of a piece
impl std::ops::Mul<Rotation> for Offset {
    type Output = Self;

    fn mul(self, rotation: Rotation) -> Self::Output {
        match rotation {
            Rotation::N => self, // no op as the coordinates are already north facing
            Rotation::S => Offset::new(-self.x, -self.y), // flip x & y axis
            Rotation::E => Offset::new(self.y, -self.x),
            Rotation::W => Offset::new(-self.y, self.x),
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
            Some([(4, 5), (4, 6), (5, 6), (5, 7)].map(Coordinate::from))
        );
    }
}
