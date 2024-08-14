use super::{color::TetriminoColor, piece::Piece, Coordinate};
use crate::engine::geometry::GridIncrement;
use std::{
    ops::{Index, IndexMut},
    slice::ArrayChunks,
};

// represents the tetris matrix
pub struct Matrix<const WIDTH: usize, const HEIGHT: usize, const SIZE: usize> {
    pub matrix: [Option<TetriminoColor>; SIZE],
}

// zero is at bottom left
impl<const WIDTH: usize, const HEIGHT: usize, const SIZE: usize> Matrix<WIDTH, HEIGHT, SIZE> {
    pub fn blank() -> Self {
        Self {
            matrix: [None; SIZE],
        }
    }

    // check whether x&y is within matrix bounds
    fn on_matrix(&self, coord: Coordinate) -> bool {
        self.valid_coord(coord) && coord.y < HEIGHT
    }

    // it's valid on the matrix or above, since a piece can be just above
    pub fn valid_coord(&self, coord: Coordinate) -> bool {
        coord.x < WIDTH
    }

    // get index in 1d array of squares in matrix
    fn indexing(Coordinate { x, y }: Coordinate) -> usize {
        y * WIDTH + x
    }

    // check if piece is either above the matrix or in a full space on the matrix
    pub fn is_clipping(&self, piece: &Piece) -> bool {
        // if some cells are None, they are clipping because they are out of bounds
        let Some(cells) = piece.cells() else {
            return true;
        };

        cells.into_iter().any(|coord| {
            !self.valid_coord(coord) || (self.on_matrix(coord) && self[coord].is_some())
        })
    }

    // check if piece is placeable on the matrix
    pub fn is_placeable(&self, piece: &Piece) -> bool {
        let Some(cells) = piece.cells() else {
            return false;
        };

        cells
            .into_iter()
            .all(|coord| self.on_matrix(coord) && self[coord].is_none())
    }

    fn is_moveable(&self, piece: &Piece) -> bool {
        let Some(cells) = piece.cells() else {
            return false;
        };

        // place all of the squares of the piece into the matrix
        cells
            .into_iter()
            .all(|coord| self.on_matrix(coord) && self[coord] == None)
    }

    // max 4 at a time because the largest piece spans only 4 lines
    pub fn clear_lines(&mut self, indices: &[usize]) {
        // sequence of removal matters - they should be removed top to bottom
        debug_assert!(indices.is_sorted());

        // iterate in reverse
        for index in indices.iter().rev() {
            let start_of_remainder = WIDTH * (index + 1); // this is the end of the range that we want to delete

            // copy over the range from the top into the existing line that we wish to remove
            self.matrix.copy_within(
                start_of_remainder.., // start of remainder to the end
                start_of_remainder - WIDTH,
            );

            // clear the top line
            self.matrix[SIZE - WIDTH..].fill(None)
        }
    }

    // returns an iterator of the slices of the lines
    fn lines(&self) -> ArrayChunks<'_, Option<TetriminoColor>, { WIDTH }> {
        self.matrix.array_chunks()
    }

    pub fn full_lines(&mut self) -> Vec<usize> {
        self.lines()
            .enumerate()
            .filter(|(_, line)| line.iter().all(Option::is_some)) // where every cell is full
            .map(|(i, _)| i) // take the indices
            .collect() // collect into the return type
    }

    pub fn clear(&mut self) {
        self.matrix[0..].fill(None)
    }
}

// implement index trait so we can index it like an array
impl<const WIDTH: usize, const HEIGHT: usize, const SIZE: usize> Index<Coordinate>
    for Matrix<WIDTH, HEIGHT, SIZE>
{
    type Output = Option<TetriminoColor>;

    fn index(&self, coord: Coordinate) -> &Self::Output {
        assert!(self.on_matrix(coord));
        &self.matrix[Self::indexing(coord)] // self.0 -> first element of a tuple
    }
}

// will return !reference! to cell (not copy of the value) if it is in bounds
impl<const WIDTH: usize, const HEIGHT: usize, const SIZE: usize> IndexMut<Coordinate>
    for Matrix<WIDTH, HEIGHT, SIZE>
{
    fn index_mut(&mut self, coord: Coordinate) -> &mut Self::Output {
        assert!(self.on_matrix(coord));
        &mut self.matrix[Self::indexing(coord)] // self.0 -> first element of a tuple
    }
}

// 'matrix is a lifetime parameter
pub struct CellIter<'matrix, const WIDTH: usize, const HEIGHT: usize, const SIZE: usize> {
    pub position: Coordinate, // starts at the bottom and goes up, tracks where we are in the iteration
    // we introduce a new lifetime, because we're acessing memory of matrix with &Option<Color>
    pub cells: ::std::slice::Iter<'matrix, Option<TetriminoColor>>,
}

impl<'matrix, const WIDTH: usize, const HEIGHT: usize, const SIZE: usize> Iterator
    for CellIter<'matrix, WIDTH, HEIGHT, SIZE>
{
    type Item = (Coordinate, Option<TetriminoColor>);

    fn next(&mut self) -> Option<Self::Item> {
        let Some(&cell) = self.cells.next() else {
            return None;
        };

        let coord = self.position;

        // grid increment the position as we've defined in geometry mod
        // self.position.grid_inc();
        <Coordinate as GridIncrement<WIDTH>>::grid_inc(&mut self.position);

        // increment the position
        Some((coord, cell))
    }
}
