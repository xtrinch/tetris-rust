use core::range::Range;
use std::ops::{Index, IndexMut};
use std::time::Duration;

use cgmath::{EuclideanSpace, Point2, Vector2};
use geometry::GridIncrement;
use piece::{Piece, Rotation};
use piece_kind::PieceKind;
use rand::prelude::SliceRandom;
use rand::rngs::ThreadRng;
use rand::thread_rng;
use std::slice::ArrayChunks;

mod geometry;
pub mod piece;
mod piece_kind;

pub type Coordinate = Point2<usize>;
type Offset = Vector2<isize>;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum MoveKind {
    Left,
    Right,
}

impl MoveKind {
    fn offset(&self) -> Offset {
        match self {
            MoveKind::Left => Offset::new(-1, 0),
            MoveKind::Right => Offset::new(1, 0),
        }
    }
}

// represents the game engine
pub struct Engine {
    matrix: Matrix,
    bag: Vec<PieceKind>, // this is from where tetris piece types are taken from during gameplay (7 are shuffled, taken out one by one, then process repeats)
    rng: ThreadRng,      // random number generator instance
    cursor: Option<Piece>, // current active piece (the one falling down), optional
    level: u8,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            matrix: Matrix::blank(),
            bag: Vec::new(),
            rng: thread_rng(),
            cursor: None,
            level: 1,
        }
    }

    pub fn with_matrix(matrix: Matrix) -> Self {
        Self {
            matrix,
            ..Self::new()
        }
    }

    // once bag where we pick new pieces from is empty, we need to refill it
    fn refill_bag(&mut self, // mutable reference to self because we are modifying ourselves
    ) {
        debug_assert!(self.bag.is_empty()); // throw if bag is not empty

        // put all pieces in bag
        self.bag.extend_from_slice(PieceKind::ALL.as_slice()); // array to slice

        // shuffle the bag
        self.bag.shuffle(&mut self.rng)
    }

    // place the cursor into the matrix onto the position it's currently at
    pub fn place_cursor(&mut self) {
        let cursor = self.cursor.unwrap();

        // validate that the piece does not overlap with any other pieces
        debug_assert!(
            self.matrix.is_placeable(&cursor),
            "Tried to place cursor in an unplaceable location: {:?}",
            cursor
        );

        let color = cursor.kind.color();
        // place all of the squares of the piece into the matrix
        for coord in cursor.cells().unwrap() {
            self.matrix[coord] = Some(color);
        }

        // self.cursor = None // reset the cursor since we've placed it
        // self.create_top_cursor();
    }

    // place the cursor into the matrix onto the position it's currently at
    fn try_place_cursor(&mut self) {
        if let Some(cursor) = self.cursor {
            self.place_cursor();
        } else {
            println!("Tried placing a nonexistant cursor")
        }
    }

    // returns Ok(()), Err(()) of unit, represented in memory same as a bool
    pub fn move_cursor(&mut self, kind: MoveKind) -> Result<(), ()> {
        let Some(cursor) = self.cursor.as_mut() else {
            return Ok(()); // because it's OK to move a cursor that isn't there, it would just do nothing
        };

        let new = cursor.moved_by(kind.offset());

        // check if it is not within moveable bounds (or above)
        if self.matrix.is_clipping(&new) {
            // TODO: check
            return Err(());
        }

        self.cursor = Some(new);
        Ok(())
    }

    pub fn rotate_cursor(&mut self, kind: Rotation) -> () {
        let Some(cursor) = self.cursor.as_mut() else {
            return; // because it's OK to move a cursor that isn't there, it would just do nothing
        };

        cursor.rotation = kind;
    }

    pub fn cursor_info(&self) -> Option<([Coordinate; Piece::CELL_COUNT], Color, Rotation)> {
        let cursor = self.cursor?; // early return a None if it was None
        Some((
            cursor.cells().unwrap(),
            cursor.kind.color(),
            cursor.rotation,
        ))
    }

    // current cursor rotation
    pub fn next_cursor_rotation(&self) -> Option<Rotation> {
        let cursor = self.cursor?; // early return a None if it was None

        Some(cursor.rotation.next_rotation())
    }

    pub fn DEBUG_test_cursor_local(&mut self, kind: PieceKind, position: Offset) {
        let piece = Piece {
            kind,
            rotation: Rotation::N,
            position,
        };
        self.cursor = Some(piece)
    }

    // creates a random tetrimino and places it above the matrix
    pub fn create_top_cursor(&mut self) {
        let kind: PieceKind = rand::random(); // we can do this because we implemented the distribution trait for this enum!

        let rotation = Rotation::N;
        let position: Offset = (4, 19).into();

        let piece = Piece {
            kind,
            rotation,
            position,
        };
        self.cursor = Some(piece)
    }

    // ticks down the cursor for one spot and if it can't, returns an error and allow extended placement
    // two ways this can fail -> hit the bottom (cells() will return None) or hit another piece
    pub fn try_tick_down(&mut self) {
        // extract cursor from the optional
        let cursor = self
            .cursor
            .as_ref()
            .expect("Tried to tick an absent cursor");

        // if cursor hit bottom, panic
        debug_assert!(!self.cursor_has_hit_bottom());

        // unwrap to catch errors
        self.cursor = Some(self.ticked_down_cursor().unwrap());
    }

    pub fn cursor_has_hit_bottom(&self) -> bool {
        self.cursor.is_some() && self.ticked_down_cursor().is_none()
    }

    // // whether the cursor is at the bottom
    // pub fn is_lock_down(&self) -> bool {}

    // get the new cursor if it was ticked down
    pub fn ticked_down_cursor(&self) -> Option<Piece> {
        let Some(cursor) = self.cursor else {
            return None;
        };
        let new = cursor.moved_by(Offset::new(0, -1));

        (!self.matrix.is_clipping(&new)).then_some(new)
    }

    // moves cursor down and places it (series of tick downs), always succeeds
    pub fn hard_drop(&mut self) {
        // while we have a ticked down cursor, move it down
        while let Some(new) = self.ticked_down_cursor() {
            self.cursor = Some(new);
        }

        // since we could press keyboard multiple times during one tick cycle, we need to not panic if there's no cursor
        self.try_place_cursor();
        self.create_top_cursor();
    }

    // get an iterator for the cells of the matrix
    pub fn cells(&self) -> CellIter<'_> {
        // '_ means a deduced lifetime, will associate matrix's lifetime with the cell iter lifetime
        CellIter {
            position: Coordinate::origin(),
            cells: self.matrix.0.iter(), // iter over first element of tuple which is our matrix array
        }
    }

    // how long the tetrimino should drop for a certain level
    pub fn drop_time(&self) -> Duration {
        // equation from the docs: (0.8 - ((level - 1) * 0.007))^(level-1)
        let level_index = self.level + 1;
        let seconds_per_line = (0.8 - ((level_index) as f32 * 0.007)).powi(level_index as i32);
        Duration::from_secs_f32(seconds_per_line)
    }

    // when a line is full, it needs to be removed from the screen
    pub fn line_clear(&mut self, mut animation: impl FnMut(&[usize])) {
        // identify full lines
        let lines: Vec<usize> = self.matrix.full_lines();

        // runs the animation of the removal of those lines
        animation(lines.as_slice());

        self.matrix.clear_lines(lines.as_slice());
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Color {
    Yellow,
    Cyan,
    Purple,
    Orange,
    Blue,
    Green,
    Red,
}
// represents the tetris matrix
pub struct Matrix([Option<Color>; Self::SIZE]);

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
    fn valid_coord(coord: Coordinate) -> bool {
        coord.x < Self::WIDTH
    }

    // get index in 1d array of squares in matrix
    fn indexing(Coordinate { x, y }: Coordinate) -> usize {
        y * Self::WIDTH + x
    }

    // check if piece is either above the matrix or in a full space on the matrix
    fn is_clipping(&self, piece: &Piece) -> bool {
        // if some cells are None, they are clipping because they are out of bounds
        let Some(cells) = piece.cells() else {
            return true;
        };

        cells.into_iter().any(|coord| {
            !Matrix::valid_coord(coord) || (Matrix::on_matrix(coord) && self[coord].is_some())
        })
    }

    // check if piece is placeable on the matrix
    fn is_placeable(&self, piece: &Piece) -> bool {
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
    fn clear_lines(&mut self, indices: &[usize]) {
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
    fn lines(&self) -> ArrayChunks<'_, Option<Color>, { Self::WIDTH }> {
        self.0.array_chunks()
    }

    fn full_lines(&mut self) -> Vec<usize> {
        self.lines()
            .enumerate()
            .filter(|(_, line)| line.iter().all(Option::is_some)) // where every cell is full
            .map(|(i, _)| i) // take the indices
            .collect() // collect into the return type
    }
}

// implement index trait so we can index it like an array
impl Index<Coordinate> for Matrix {
    type Output = Option<Color>;

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
    position: Coordinate, // starts at the bottom and goes up, tracks where we are in the iteration
    // we introduce a new lifetime, because we're acessing memory of matrix with &Option<Color>
    cells: ::std::slice::Iter<'matrix, Option<Color>>,
}

impl<'matrix> Iterator for CellIter<'matrix> {
    type Item = (Coordinate, Option<Color>);

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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn cell_iter() {
        let mut matrix = Matrix::blank();
        matrix[Coordinate::new(2, 0)] = Some(Color::Blue);
        matrix[Coordinate::new(3, 1)] = Some(Color::Green);

        let mut iter = CellIter {
            position: Coordinate::origin(),
            cells: matrix.0.iter(), // iter over first element of tuple which is our matrix array
        };

        let first_five = (&mut iter).take(5).collect::<Vec<_>>();
        assert_eq!(
            first_five,
            [
                (Coordinate::new(0, 0), None),
                (Coordinate::new(1, 0), None),
                (Coordinate::new(2, 0), Some(Color::Blue)),
                (Coordinate::new(3, 0), None),
                (Coordinate::new(4, 0), None)
            ]
        );

        let other_item = (&mut iter).skip(8).next();
        assert_eq!(
            other_item,
            Some((Coordinate::new(3, 1), Some(Color::Green)))
        );

        assert!(iter.all(|(_, contents)| contents.is_none()));
    }
}
