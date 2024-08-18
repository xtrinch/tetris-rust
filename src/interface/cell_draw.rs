use crate::interface::render_traits::ScreenColor;
use cgmath::ElementWise;
use cgmath::EuclideanSpace;
use cgmath::{Point2, Vector2};
use sdl2::{pixels::Color, rect::Rect, render::Canvas, video::Window};

use crate::engine::{
    color::TetriminoColor,
    matrix::{CellIter, Matrix},
    Coordinate,
};

// we need a lifetime because we have a mutable reference
pub struct CellDrawContext<'canvas, const WIDTH: usize, const HEIGHT: usize>
where
    [usize; WIDTH * HEIGHT]:,
{
    pub origin: Point2<i32>,
    pub dims: Vector2<u32>,
    pub canvas: &'canvas mut Canvas<Window>,
    pub matrix: &'canvas Matrix<WIDTH, HEIGHT>,
}

impl<const WIDTH: usize, const HEIGHT: usize> CellDrawContext<'_, { WIDTH }, { HEIGHT }>
where
    [usize; WIDTH * HEIGHT]:,
{
    const CELL_COUNT: Vector2<u32> = Vector2::new(WIDTH as u32, HEIGHT as u32);

    pub fn draw_matrix(&mut self) {
        let cell_iter: CellIter<WIDTH, HEIGHT> = CellIter {
            position: Coordinate::origin(),
            cells: self.matrix.matrix.iter(), // iter over first element of tuple which is our matrix array
        };

        for (coord, _) in cell_iter {
            self.draw_border(coord);
        }

        let cell_iter1: CellIter<WIDTH, HEIGHT> = CellIter {
            position: Coordinate::origin(),
            cells: self.matrix.matrix.iter(), // iter over first element of tuple which is our matrix array
        };

        for (coord, cell) in cell_iter1 {
            self.try_draw_cell(coord, cell);
        }
    }

    fn get_rect(&mut self, coord: Coordinate) -> Rect {
        // // we get the width from the next cells coordinates because otherwise we end up with a rounding error
        // let this_x = (coord.x as u32 + 0) * matrix_width / Matrix::WIDTH as u32;
        // let this_y = (coord.y as u32 + 1) * matrix_height / Matrix::HEIGHT as u32;

        // let next_x = (coord.x as u32 + 1) * matrix_width / Matrix::WIDTH as u32;
        // let prev_y = (coord.y as u32 + 0) * matrix_height / Matrix::HEIGHT as u32; // we take the previous y because that one will be ABOVE it

        // this is just a more complex version of the thing above which is much easier to understand

        let coord = coord.to_vec().cast::<u32>().unwrap();
        let this = (coord + Vector2::new(0, 1))
            .mul_element_wise(self.dims)
            .div_element_wise(Self::CELL_COUNT);
        let next = (coord + Vector2::new(1, 0))
            .mul_element_wise(self.dims)
            .div_element_wise(Self::CELL_COUNT);

        // our matrix goes bottom left +, their draw matrix goes from top left +, so we need to do some translation
        Rect::new(
            self.origin.x + this.x as i32,
            self.origin.y - this.y as i32 - 1, // we subtract so we go up instead of down since origin is top left for the draw matrix (we also add one since the rect is drawn in the opposite direction); -1 is because we do border overlap adjustments
            next.x - this.x + 1, // next x is "to the right", -1 to make the borders overlap
            this.y - next.y + 1, // prev_y is "higher", -1 to make the borders overlap
        )
    }

    pub fn try_draw_cell(&mut self, coord: Coordinate, cell: Option<TetriminoColor>) {
        let Some(color) = cell else {
            return;
        };

        let cell_rect = self.get_rect(coord);

        self.canvas.set_draw_color(color.screen_color());
        self.canvas.fill_rect(cell_rect).unwrap();

        self.canvas.set_draw_color(Color::WHITE);
        self.canvas.draw_rect(cell_rect).unwrap();
    }

    fn draw_border(&mut self, coord: Coordinate) {
        let cell_rect = self.get_rect(coord);

        self.canvas.set_draw_color(Color::RGB(130, 130, 130));
        self.canvas.draw_rect(cell_rect).unwrap();
    }
}
