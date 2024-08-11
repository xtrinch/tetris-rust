use std::ops::{Index, IndexMut};

use cgmath::{EuclideanSpace, Point2, Vector2};
use geometry::GridIncrement;
use piece::{Kind as PieceKind, Piece};
use rand::prelude::SliceRandom;
use rand::rngs::ThreadRng;
use rand::thread_rng;

mod geometry;
mod piece;

type Coordinate = Point2<usize>;
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
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            matrix: Matrix::blank(),
            bag: Vec::new(),
            rng: thread_rng(),
            cursor: None,
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
    fn place_cursor(&mut self) {
        let cursor = self
            .cursor
            .take()
            .expect("Called place_cursor without a cursor");

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
    }

    // returns Ok(()), Err(()) of unit, represented in memory same as a bool
    fn move_cursor(&mut self, kind: MoveKind) -> Result<(), ()> {
        let Some(cursor) = self.cursor.as_mut() else {
            return Ok(()); // because it's OK to move a cursor that isn't there, it would just do nothing
        };

        let new = cursor.moved_by(kind.offset());

        // check if it is not within moveable bounds (or above)
        if self.matrix.is_clipping(&new) {
            // TODO: check
            return Err(());
        }

        Ok(self.cursor = Some(new))
    }

    // ticks down the cursor for one spot and if it can't, returns an error and allow extended placement
    // two ways this can fail -> hit the bottom (cells() will return None) or hit another piece
    fn try_tick_down(&mut self) {
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

    // get the new cursor if it was ticked down
    fn ticked_down_cursor(&self) -> Option<Piece> {
        let Some(cursor) = self.cursor else {
            return None;
        };
        let new = cursor.moved_by(Offset::new(0, -1));

        (!self.matrix.is_clipping(&new)).then_some(new)
    }

    // moves cursor down and places it (series of tick downs), always succeeds
    fn hard_drop(&mut self) {
        // while we have a ticked down cursor, move it down
        while let Some(new) = self.ticked_down_cursor() {
            self.cursor = Some(new);
        }

        self.place_cursor()
    }

    // get an iterator for the cells of the matrix
    pub fn cells(&self) -> CellIter<'_> {
        // '_ means a deduced lifetime, will associate matrix's lifetime with the cell iter lifetime
        CellIter {
            position: Coordinate::origin(),
            cells: self.matrix.0.iter(), // iter over first element of tuple which is our matrix array
        }
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

    fn blank() -> Self {
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

    // check if piece is either above the matrix or in empty space on the matrix
    fn is_clipping(&self, piece: &Piece) -> bool {
        // if some cells are None, they are clipping because they are out of bounds
        let Some(cells) = piece.cells() else {
            return true;
        };

        cells
            .into_iter()
            .any(|coord| !Matrix::on_matrix(coord) || self[coord].is_some())
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
    type Item = (Coordinate, &'matrix Option<Color>);

    fn next(&mut self) -> Option<Self::Item> {
        let Some(cell) = self.cells.next() else {
            return None;
        };

        let coord = self.position;

        // grid increment the position as we've defined in geometry mod
        self.position.grid_inc();

        // increment the position
        return Some((coord, cell));
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
                (Coordinate::new(0, 0), &None),
                (Coordinate::new(1, 0), &None),
                (Coordinate::new(2, 0), &Some(Color::Blue)),
                (Coordinate::new(3, 0), &None),
                (Coordinate::new(4, 0), &None)
            ]
        );

        let other_item = (&mut iter).skip(8).next();
        assert_eq!(
            other_item,
            Some((Coordinate::new(3, 1), &Some(Color::Green)))
        );

        assert!(iter.all(|(_, contents)| contents.is_none()));
    }
}
