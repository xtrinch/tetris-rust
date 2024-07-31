use std::ops::{Index, IndexMut};

use cgmath::Vector2;
use piece::{Kind as PieceKind, Piece};
use rand::prelude::SliceRandom;
use rand::rngs::ThreadRng;
use rand::thread_rng;

mod piece;

type Coordinate = Vector2<usize>;
type Offset = Vector2<isize>;

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

    // place the cursor into the matrix (on top)
    fn place_cursor(&mut self) {
        let cursor = self
            .cursor
            .take()
            .expect("Called place_cursor without a cursor");

        // place all of the squares of the piece into the matrix
        for coord in cursor.cells().expect("Cursor was out of bounds") {
            let cell = &mut self.matrix[coord];

            // validate that the piece does not overlap with any other pieces
            debug_assert_eq!(*cell, false);

            // reassign cell
            *cell = true;
        }
    }
}

// represents the tetris matrix
struct Matrix([bool; Self::SIZE]);

impl Matrix {
    const WIDTH: usize = 10; // matrix 10 cells wide
    const HEIGHT: usize = 20; // matrix 20 cells high
    const SIZE: usize = Self::WIDTH * Self::HEIGHT;

    fn blank() -> Self {
        Self([false; Self::SIZE])
    }

    // check whether x&y is within matrix bounds
    fn in_bounds(Coordinate { x, y }: Coordinate) -> bool {
        x < Self::WIDTH && y < Self::HEIGHT
    }

    // get index in 1d array of squares in matrix
    fn indexing(Coordinate { x, y }: Coordinate) -> usize {
        y * Self::WIDTH + x
    }
}

impl Index<Coordinate> for Matrix {
    type Output = bool;

    fn index(&self, coord: Coordinate) -> &Self::Output {
        assert!(Self::in_bounds(coord));
        &self.0[Self::indexing(coord)] // self.0 -> first element of a tuple
    }
}

// will return !reference! to cell (not copy of the value) if it is in bounds
impl IndexMut<Coordinate> for Matrix {
    fn index_mut(&mut self, coord: Coordinate) -> &mut Self::Output {
        assert!(Self::in_bounds(coord));
        &mut self.0[Self::indexing(coord)] // self.0 -> first element of a tuple
    }
}
