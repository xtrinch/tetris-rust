use cgmath::Zero;

use super::piece_kind::PieceKind;
use super::piece_rotation::Rotation;
use super::{Coordinate, Offset};
use super::{Engine, Matrix};
use cgmath::EuclideanSpace;

#[derive(Clone, Copy, PartialEq, Debug)]

pub struct Piece {
    pub kind: PieceKind,  // these do not need to be in a constructor
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

            // TODO: reintroduce check?
            // // check that the position is within bounds, the negative check is already done by the conversion above
            // if self.valid_coord(coord) {
            *coord_slot = coord;
            // } else {
            //     return None;
            // }
        }

        Some(coords)
    }

    // will rotate a single tetrimino
    fn rotator(&self) -> impl Fn(Offset) -> Offset + '_ {
        // to capture lifetime of self, add '_
        |cell| match self.kind {
            PieceKind::O => cell, // skip rotation for square as it's defined as 3x3 and it's not rotated
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

#[cfg(test)]
mod test {
    use crate::engine::piece_rotation::Rotation;

    use super::*;

    #[test]
    fn s_piece_positioning() {
        let z = Piece {
            kind: PieceKind::Z,
            position: Offset::new(5, 6),
            rotation: Rotation::W,
        };
        assert_eq!(
            z.cells(),
            Some([(5, 6), (5, 7), (6, 7), (6, 8)].map(Coordinate::from))
        );
    }
}
