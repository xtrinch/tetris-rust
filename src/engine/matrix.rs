use super::{color::TetriminoColor, piece::Piece, Coordinate};
use crate::engine::geometry::GridIncrement;
use std::{
    ops::{Index, IndexMut},
    slice::ArrayChunks,
};

// represents the tetris matrix
pub struct Matrix(pub [Option<TetriminoColor>; Self::SIZE]);

// zero is at bottom left
impl Matrix {
    pub const WIDTH: usize = 10; // matrix 10 cells wide
    pub const HEIGHT: usize = 20; // matrix 20 cells high
    pub const SIZE: usize = Self::WIDTH * Self::HEIGHT;

    pub fn blank() -> Self {
        Self([None; Self::SIZE])
    }

    // check whether x&y is within matrix bounds
    fn on_matrix(coord: Coordinate) -> bool {
        Self::valid_coord(coord) && coord.y < Self::HEIGHT
    }

    // it's valid on the matrix or above, since a piece can be just above
    pub fn valid_coord(coord: Coordinate) -> bool {
        coord.x < Self::WIDTH
    }

    // get index in 1d array of squares in matrix
    fn indexing(Coordinate { x, y }: Coordinate) -> usize {
        y * Self::WIDTH + x
    }

    // check if piece is either above the matrix or in a full space on the matrix
    pub fn is_clipping(&self, piece: &Piece) -> bool {
        // if some cells are None, they are clipping because they are out of bounds
        let Some(cells) = piece.cells() else {
            return true;
        };

        cells.into_iter().any(|coord| {
            !Matrix::valid_coord(coord) || (Matrix::on_matrix(coord) && self[coord].is_some())
        })
    }

    // check if piece is placeable on the matrix
    pub fn is_placeable(&self, piece: &Piece) -> bool {
        let Some(cells) = piece.cells() else {
            return false;
        };

        cells
            .into_iter()
            .all(|coord| Matrix::on_matrix(coord) && self[coord].is_none())
    }

    fn is_moveable(&self, piece: &Piece) -> bool {
        let Some(cells) = piece.cells() else {
            return false;
        };

        // place all of the squares of the piece into the matrix
        cells
            .into_iter()
            .all(|coord| Matrix::on_matrix(coord) && self[coord] == None)
    }

    // max 4 at a time because the largest piece spans only 4 lines
    pub fn clear_lines(&mut self, indices: &[usize]) {
        // sequence of removal matters - they should be removed top to bottom
        debug_assert!(indices.is_sorted());

        // iterate in reverse
        for index in indices.iter().rev() {
            let start_of_remainder = Self::WIDTH * (index + 1); // this is the end of the range that we want to delete

            // copy over the range from the top into the existing line that we wish to remove
            self.0.copy_within(
                start_of_remainder.., // start of remainder to the end
                start_of_remainder - Self::WIDTH,
            );

            // clear the top line
            self.0[Self::SIZE - Self::WIDTH..].fill(None)
        }
    }

    // returns an iterator of the slices of the lines
    fn lines(&self) -> ArrayChunks<'_, Option<TetriminoColor>, { Self::WIDTH }> {
        self.0.array_chunks()
    }

    pub fn full_lines(&mut self) -> Vec<usize> {
        self.lines()
            .enumerate()
            .filter(|(_, line)| line.iter().all(Option::is_some)) // where every cell is full
            .map(|(i, _)| i) // take the indices
            .collect() // collect into the return type
    }
}

// implement index trait so we can index it like an array
impl Index<Coordinate> for Matrix {
    type Output = Option<TetriminoColor>;

    fn index(&self, coord: Coordinate) -> &Self::Output {
        assert!(Self::on_matrix(coord));
        &self.0[Self::indexing(coord)] // self.0 -> first element of a tuple
    }
}

// will return !reference! to cell (not copy of the value) if it is in bounds
impl IndexMut<Coordinate> for Matrix {
    fn index_mut(&mut self, coord: Coordinate) -> &mut Self::Output {
        assert!(Self::on_matrix(coord));
        &mut self.0[Self::indexing(coord)] // self.0 -> first element of a tuple
    }
}

// 'matrix is a lifetime parameter
pub struct CellIter<'matrix> {
    pub position: Coordinate, // starts at the bottom and goes up, tracks where we are in the iteration
    // we introduce a new lifetime, because we're acessing memory of matrix with &Option<Color>
    pub cells: ::std::slice::Iter<'matrix, Option<TetriminoColor>>,
}

impl<'matrix> Iterator for CellIter<'matrix> {
    type Item = (Coordinate, Option<TetriminoColor>);

    fn next(&mut self) -> Option<Self::Item> {
        let Some(&cell) = self.cells.next() else {
            return None;
        };

        let coord = self.position;

        // grid increment the position as we've defined in geometry mod
        self.position.grid_inc();

        // increment the position
        Some((coord, cell))
    }
}
