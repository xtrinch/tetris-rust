use std::time::Duration;

use cgmath::{EuclideanSpace, Point2, Vector2};
use color::TetriminoColor;
use matrix::{CellIter, Matrix};
use move_kind::MoveKind;
use piece::Piece;
use piece_kind::PieceKind;
use piece_rotation::Rotation;
use rand::prelude::SliceRandom;
use rand::rngs::ThreadRng;
use rand::thread_rng;

pub mod color;
mod geometry;
pub mod matrix;
pub mod move_kind;
pub mod piece;
mod piece_kind;
pub mod piece_rotation;

pub type Coordinate = Point2<usize>;
type Offset = Vector2<isize>;

// represents the game engine
pub struct Engine {
    pub matrix: Matrix<{ Self::MATRIX_WIDTH }, { Self::MATRIX_HEIGHT }>,
    pub up_next_matrix:
        Matrix<{ Self::SINGLE_TETRIMINO_MATRIX_WIDTH }, { Self::SINGLE_TETRIMINO_MATRIX_HEIGHT }>,
    pub hold_matrix:
        Matrix<{ Self::SINGLE_TETRIMINO_MATRIX_WIDTH }, { Self::SINGLE_TETRIMINO_MATRIX_HEIGHT }>,
    pub queue_matrix:
        Matrix<{ Self::REMAINING_NEXT_MATRIX_WIDTH }, { Self::REMAINING_NEXT_MATRIX_HEIGHT }>,
    next: Vec<PieceKind>, // next up, these are also visible on the screen (7), they are filled from the bag or randomly
    bag: Vec<PieceKind>, // this is from where tetris piece types are taken from during gameplay (7 are shuffled, taken out one by one, then process repeats)
    hold: Option<PieceKind>,
    rng: ThreadRng,        // random number generator instance
    cursor: Option<Piece>, // current active piece (the one falling down), optional
    pub level: u8,         // fixed goal System requires 10 lines each level through level 15
    pub lines_reached: u32,
    pub score: u32, // will equal an acumulation of lines reached for the simple scoring
}

impl Engine {
    pub const MATRIX_WIDTH: usize = 10; // matrix 10 cells wide
    pub const MATRIX_HEIGHT: usize = 20; // matrix 20 cells high

    pub const SINGLE_TETRIMINO_MATRIX_WIDTH: usize = 4;
    pub const SINGLE_TETRIMINO_MATRIX_HEIGHT: usize = 4;

    pub const REMAINING_NEXT_MATRIX_WIDTH: usize = 4;
    pub const REMAINING_NEXT_MATRIX_HEIGHT: usize = 6 * 4; // 6 of the 7 items in next vector; TODO: from constant

    pub const LINES_PER_LEVEL: u32 = 10;

    pub fn new() -> Self {
        let mut rng = thread_rng();
        let mut up_next = Vec::from(PieceKind::ALL.as_slice());
        up_next.shuffle(&mut rng);

        Engine {
            matrix: Matrix::<{ Self::MATRIX_WIDTH }, { Self::MATRIX_HEIGHT }>::blank(),
            up_next_matrix: Matrix::<
                { Self::SINGLE_TETRIMINO_MATRIX_WIDTH },
                { Self::SINGLE_TETRIMINO_MATRIX_HEIGHT },
            >::blank(),
            hold_matrix: Matrix::<
                { Self::SINGLE_TETRIMINO_MATRIX_WIDTH },
                { Self::SINGLE_TETRIMINO_MATRIX_HEIGHT },
            >::blank(),
            queue_matrix: Matrix::<
                { Self::REMAINING_NEXT_MATRIX_WIDTH },
                { Self::REMAINING_NEXT_MATRIX_HEIGHT },
            >::blank(),
            bag: Vec::new(),
            next: up_next,
            rng: rng,
            cursor: None,
            hold: None,
            level: 1,
            lines_reached: 0,
            score: 0,
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

    // place the cursor into the matrix onto the position it's currently at;
    // if that's not possible, it's game over
    pub fn place_cursor(&mut self) -> bool {
        let cursor = self.cursor.unwrap();

        // // validate that the piece does not overlap with any other pieces
        // debug_assert!(
        //     self.matrix.is_placeable(&cursor),
        //     "Tried to place cursor in an unplaceable location: {:?}",
        //     cursor
        // );

        if !self.matrix.is_placeable(&cursor) {
            return false;
        }

        self.matrix.place_piece(cursor);
        return true;
    }

    // place the cursor into the matrix onto the position it's currently at; if it returns false, it's game over
    pub fn try_place_cursor(&mut self) -> bool {
        if let Some(cursor) = self.cursor {
            return self.place_cursor();
        } else {
            println!("Tried placing a nonexistant cursor")
        }

        return true;
    }

    // returns Ok(()), Err(()) of unit, represented in memory same as a bool
    pub fn move_cursor(&mut self, kind: MoveKind) -> Result<(), ()> {
        let Some(cursor) = self.cursor.as_mut() else {
            return Ok(()); // because it's OK to move a cursor that isn't there, it would just do nothing
        };

        let new = cursor.moved_by(kind.offset());

        // check if it is not within moveable bounds (or above)
        if self.matrix.is_clipping(&new) {
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

    pub fn rotate_and_adjust_cursor(&mut self, kind: Rotation) -> Option<()> {
        // check if any position is out of bounds
        let mut cursor_clone = self.cursor?.clone();

        cursor_clone.rotation = kind;

        // if cursor has out of bounds coordinates, do not rotate
        if self.matrix.has_piece_out_of_bounds_coords(&cursor_clone) {
            return None;
        }

        // otherwise perform the rotation
        self.cursor = Some(cursor_clone);

        return Some(());
    }

    pub fn cursor_info(
        &self,
    ) -> Option<([Coordinate; Piece::CELL_COUNT], TetriminoColor, Rotation)> {
        let cursor: Piece = self.cursor?; // early return a None if it was None
        let cells = self.matrix.piece_cells(&cursor)?;

        Some((cells, cursor.kind.color(), cursor.rotation))
    }

    // current cursor rotation
    pub fn next_cursor_rotation(&self) -> Option<Rotation> {
        let cursor = self.cursor?; // early return a None if it was None

        Some(cursor.rotation.next_rotation())
    }

    // creates a random tetrimino and places it above the matrix
    pub fn create_top_cursor(&mut self, force_kind: Option<PieceKind>) {
        let kind: PieceKind;
        if force_kind.is_some() {
            kind = force_kind.unwrap();
        } else {
            kind = self.next.remove(0);

            // TODO: prettify
            // add a new one since we removed one
            let new_tetrimino: PieceKind = rand::random(); // we can do this because we implemented the distribution trait for this enum!
            self.next.push(new_tetrimino);

            // readd cells in up next matrix
            self.up_next_matrix.clear();
            self.queue_matrix.clear();

            for (index, next_up) in self.next.iter().rev().enumerate() {
                let mut piece = Piece {
                    kind: *next_up,
                    position: (0, 0).into(),
                    rotation: Rotation::N,
                };

                // the up next tetrimino
                if index == self.next.len() - 1 {
                    self.up_next_matrix.place_piece(piece);
                } else {
                    // the queue tetriminos
                    let inside_index = index;
                    piece.position = (0, ((inside_index) * 4) as isize).into();

                    for coord in self.matrix.piece_cells(&piece).unwrap() {
                        // TODO: some constants
                        // add to y so we get a top-to-bottom queue
                        self.queue_matrix[(coord.x, coord.y).into()] = Some(piece.kind.color());
                    }
                }
            }
        }
        // tetriminos are all generated north facing (just as they appear in the next Queue)
        let rotation = Rotation::N;

        /*
           tetriminos are generated on the 21st and 22nd rows
           and every tetrimino that is three Minos wide is generated on the 4th cell across and stretches to the 6th.
           this includes the t-tetrimino, L-tetrimino, j-tetrimino, S-tetrimino and z-tetrimino.
           the I-tetrimino and o-tetrimino are exactly centered at generation.
           the I-tetrimino is generated on the 21st row (not 22nd), stretching from the 4th to 7th cells.
           the o-tetrimino is generated on the 5th and  6th cell.
        */

        let (mut x, mut y) = (0, 0);

        // the I-tetrimino should start lower than the rest because of its north height being smaller
        match kind.north_height() {
            2 => y = 19,
            1 => y = 18,
            _ => y = 19,
        }

        // try to center them as best we can;
        match kind.north_width() {
            2 => x = 4,
            3 => x = 3,
            4 => x = 3,
            _ => todo!(),
        }

        let position = (x, y).into();

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
    }

    pub fn try_hold(&mut self) -> Option<bool> {
        let mut cursor: Piece = self.cursor?; // early return a None if it was None

        // if we don't have a hold or the hold is not the same as the current cursor
        if self.hold.is_none() || (self.hold.is_some() && self.hold.unwrap() != cursor.kind) {
            self.hold_matrix.clear();

            let old_hold = self.hold;
            self.hold = Some(cursor.kind);
            cursor.position = (0, 0).into(); // TODO: make sure this doesn't do phantom draws of cursor at 0,0
            self.hold_matrix.place_piece(cursor);

            self.cursor = None;

            // create top cursor from whatever was on hold if there was anything
            self.create_top_cursor(old_hold);
        }

        return Some(true);
    }

    // get an iterator for the cells of the matrix
    pub fn cells(&self) -> CellIter<'_, { Self::MATRIX_WIDTH }, { Self::MATRIX_HEIGHT }> {
        // '_ means a deduced lifetime, will associate matrix's lifetime with the cell iter lifetime
        CellIter {
            position: Coordinate::origin(),
            cells: self.matrix.matrix.iter(), // iter over first element of tuple which is our matrix array
        }
    }

    // get an iterator for the cells of the matrix
    pub fn cells_up_next(
        &self,
    ) -> CellIter<
        '_,
        { Self::SINGLE_TETRIMINO_MATRIX_WIDTH },
        { Self::SINGLE_TETRIMINO_MATRIX_HEIGHT },
    > {
        // '_ means a deduced lifetime, will associate matrix's lifetime with the cell iter lifetime
        CellIter {
            position: Coordinate::origin(),
            cells: self.up_next_matrix.matrix.iter(), // iter over first element of tuple which is our matrix array
        }
    }

    // get an iterator for the cells of the matrix
    pub fn cells_hold(
        &self,
    ) -> CellIter<
        '_,
        { Self::SINGLE_TETRIMINO_MATRIX_WIDTH },
        { Self::SINGLE_TETRIMINO_MATRIX_HEIGHT },
    > {
        // '_ means a deduced lifetime, will associate matrix's lifetime with the cell iter lifetime
        CellIter {
            position: Coordinate::origin(),
            cells: self.hold_matrix.matrix.iter(), // iter over first element of tuple which is our matrix array
        }
    }

    // TODO: cleanup, do we even need this methd
    // get an iterator for the cells of the matrix
    pub fn cells_remaining_next(
        &self,
    ) -> CellIter<'_, { Self::REMAINING_NEXT_MATRIX_WIDTH }, { Self::REMAINING_NEXT_MATRIX_HEIGHT }>
    {
        // '_ means a deduced lifetime, will associate matrix's lifetime with the cell iter lifetime
        CellIter {
            position: Coordinate::origin(),
            cells: self.queue_matrix.matrix.iter(), // iter over first element of tuple which is our matrix array
        }
    }

    // how long the tetrimino should drop for a certain level
    pub fn drop_time(&self, is_soft_drop: bool) -> Duration {
        // equation from the docs: (0.8 - ((level - 1) * 0.007))^(level-1)
        let level_index = self.level + 1;
        let mut seconds_per_line = (0.8 - ((level_index) as f32 * 0.007)).powi(level_index as i32);
        if is_soft_drop {
            seconds_per_line /= 20.0;
        }
        Duration::from_secs_f32(seconds_per_line)
    }

    // when a line is full, it needs to be removed from the screen
    pub fn line_clear(&mut self, mut animation: impl FnMut(&[usize])) {
        // identify full lines
        let lines: Vec<usize> = self.matrix.full_lines();

        // runs the animation of the removal of those lines
        animation(lines.as_slice());

        self.matrix.clear_lines(lines.as_slice());

        self.lines_reached += lines.len() as u32;
        self.score += lines.len() as u32;

        if self.lines_reached >= Self::LINES_PER_LEVEL {
            self.level += 1;
            self.lines_reached = 0;
        }
    }

    pub fn reset(&mut self) {
        self.cursor = None;
        self.matrix.clear();
        self.level = 1;
        self.score = 0;
        self.lines_reached = 0;
    }
}

#[cfg(test)]
mod test {
    use matrix::CellIter;

    use super::*;

    #[test]
    fn cell_iter() {
        let mut matrix = Matrix::<10, 20>::blank();
        matrix[Coordinate::new(2, 0)] = Some(TetriminoColor::Blue);
        matrix[Coordinate::new(3, 1)] = Some(TetriminoColor::Green);

        let mut iter: CellIter<10, 20> = CellIter {
            position: Coordinate::origin(),
            cells: matrix.matrix.iter(), // iter over first element of tuple which is our matrix array
        };

        let first_five = (&mut iter).take(5).collect::<Vec<_>>();
        assert_eq!(
            first_five,
            [
                (Coordinate::new(0, 0), None),
                (Coordinate::new(1, 0), None),
                (Coordinate::new(2, 0), Some(TetriminoColor::Blue)),
                (Coordinate::new(3, 0), None),
                (Coordinate::new(4, 0), None)
            ]
        );

        let other_item = (&mut iter).skip(8).next();
        assert_eq!(
            other_item,
            Some((Coordinate::new(3, 1), Some(TetriminoColor::Green)))
        );

        assert!(iter.all(|(_, contents)| contents.is_none()));
    }
}
