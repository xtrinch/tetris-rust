use super::piece_kind::PieceKind;
use super::piece_rotation::Rotation;
use super::Offset;

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

    // will rotate a single tetrimino
    pub fn rotator(&self) -> impl Fn(Offset) -> Offset + '_ {
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
    pub fn positioner(&self) -> impl Fn(Offset) -> Offset {
        let position: Offset = self.position;
        move |cell: Offset| cell + position // move it into address space
    }

    // non matrix-specific offsets; these can be negative and even out of bounds of the matrix
    pub fn matrix_offsets(&self) -> [Offset; Self::CELL_COUNT] {
        // array of 4 offsets which we need to convert into coordinates
        let offsets = self.kind.cells().map(self.rotator()).map(self.positioner());

        offsets
    }
}

// #[cfg(test)]
// mod test {
//     use crate::engine::piece_rotation::Rotation;

//     use super::*;

//     #[test]
//     fn s_piece_positioning() {
//         let z = Piece {
//             kind: PieceKind::Z,
//             position: Offset::new(5, 6),
//             rotation: Rotation::W,
//         };
//         assert_eq!(
//             z.cells(),
//             Some([(5, 6), (5, 7), (6, 7), (6, 8)].map(Coordinate::from))
//         );
//     }
// }
