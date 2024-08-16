use super::{color::TetriminoColor, piece::Piece, Coordinate, Offset};
use crate::engine::geometry::GridIncrement;
use cgmath::EuclideanSpace;
use std::{
    ops::{Index, IndexMut},
    slice::ArrayChunks,
};

// represents the tetris matrix
pub struct Matrix<const WIDTH: usize, const HEIGHT: usize>
where
    [usize; WIDTH * HEIGHT]:,
{
    pub matrix: [Option<TetriminoColor>; WIDTH * HEIGHT],
}

// zero is at bottom left
impl<const WIDTH: usize, const HEIGHT: usize> Matrix<WIDTH, HEIGHT>
where
    [usize; WIDTH * HEIGHT]:,
{
    const SIZE: usize = WIDTH * HEIGHT;

    pub fn blank() -> Self {
        Self {
            matrix: [None; WIDTH * HEIGHT],
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
        let Some(cells) = self.piece_cells(piece) else {
            return true;
        };

        cells.into_iter().any(|coord| {
            !self.valid_coord(coord) || (self.on_matrix(coord) && self[coord].is_some())
        })
    }

    // check if piece is placeable on the matrix
    pub fn is_placeable(&self, piece: &Piece) -> bool {
        let Some(cells) = self.piece_cells(piece) else {
            return false;
        };

        cells
            .into_iter()
            .all(|coord| self.on_matrix(coord) && self[coord].is_none())
    }

    fn is_moveable(&self, piece: &Piece) -> bool {
        let Some(cells) = self.piece_cells(piece) else {
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
            self.matrix[Self::SIZE - WIDTH..].fill(None)
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

    // place all of the squares of the piece into the matrix
    pub fn place_piece(&mut self, piece: Piece) {
        let color: TetriminoColor = piece.kind.color();

        for coord in self.piece_cells(&piece).unwrap() {
            self[coord] = Some(color);
        }
    }

    pub fn has_piece_out_of_bounds_coords(&self, piece: &Piece) -> bool {
        piece.matrix_offsets().into_iter().any(|coord| {
            let is_invalid = coord[0] < 0 || coord[1] < 0 || coord[0] >= WIDTH as isize;
            if is_invalid {
                return true;
            }

            // cast to something we can index the matrix with
            let positive_offset = coord.cast::<usize>().unwrap(); // the question mark denotes that if this returns none, the whole thing will return none
            let coord = Coordinate::from_vec(positive_offset);
            if self.on_matrix(coord) && self[coord].is_some() {
                // it's on the matrix and overlapping
                return true;
            }

            false
        })
    }

    // returns coordinates of piece; None on an invalid cursor position;
    // returns an array of length CELL_COUNT
    pub fn piece_cells(&self, piece: &Piece) -> Option<[Coordinate; Piece::CELL_COUNT]> {
        // array of 4 offsets which we need to convert into coordinates
        let offsets = piece.matrix_offsets();

        let mut coords = [Coordinate::origin(); Piece::CELL_COUNT];

        // convert to coords
        for (offset, coord_slot) in offsets.into_iter().zip(&mut coords) {
            // cast to a positive integer and let it throw if it can't be
            let positive_offset = offset.cast::<usize>()?; // the question mark denotes that if this returns none, the whole thing will return none
            let coord = Coordinate::from_vec(positive_offset);

            // check that the position is within bounds, the negative check is already done by the conversion above
            if self.valid_coord(coord) {
                *coord_slot = coord;
            } else {
                return None;
            }
        }

        Some(coords)
    }
}

// implement index trait so we can index it like an array
impl<const WIDTH: usize, const HEIGHT: usize> Index<Coordinate> for Matrix<WIDTH, HEIGHT>
where
    [usize; WIDTH * HEIGHT]:,
{
    type Output = Option<TetriminoColor>;

    fn index(&self, coord: Coordinate) -> &Self::Output {
        assert!(self.on_matrix(coord));
        &self.matrix[Self::indexing(coord)] // self.0 -> first element of a tuple
    }
}

// will return !reference! to cell (not copy of the value) if it is in bounds
impl<const WIDTH: usize, const HEIGHT: usize> IndexMut<Coordinate> for Matrix<WIDTH, HEIGHT>
where
    [usize; WIDTH * HEIGHT]:,
{
    fn index_mut(&mut self, coord: Coordinate) -> &mut Self::Output {
        assert!(self.on_matrix(coord));
        &mut self.matrix[Self::indexing(coord)] // self.0 -> first element of a tuple
    }
}

// 'matrix is a lifetime parameter
pub struct CellIter<'matrix, const WIDTH: usize, const HEIGHT: usize> {
    pub position: Coordinate, // starts at the bottom and goes up, tracks where we are in the iteration
    // we introduce a new lifetime, because we're acessing memory of matrix with &Option<Color>
    pub cells: ::std::slice::Iter<'matrix, Option<TetriminoColor>>,
}

impl<'matrix, const WIDTH: usize, const HEIGHT: usize> Iterator
    for CellIter<'matrix, WIDTH, HEIGHT>
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
