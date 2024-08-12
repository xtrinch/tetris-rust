use crate::engine::{Engine, Matrix};
use crate::interface::render_traits::ScreenColor;
use cgmath::{ElementWise, EuclideanSpace, Vector2};
use sdl2::{
    event::Event,
    pixels::Color,
    rect::{Point, Rect},
    render::Canvas,
    video::Window,
};
use std::cmp::min;

mod render_traits;

pub struct Interface {
    engine: Engine,
}

const INIT_SIZE: Vector2<u32> = Vector2::new(1024, 1024);
const BACKGROUND_COLOR: Color = Color::RGB(0x10, 0x10, 0x18);
const MATRIX_COLOR: Color = Color::RGB(0x66, 0x77, 0x77);
const PLACEHOLDER_2: Color = Color::RGB(0x66, 0x77, 0x77);
const PLACEHOLDER_3: Color = Color::RGB(0x77, 0x88, 0x88);

impl Interface {
    pub fn run(engine: Engine) {
        let sdl = sdl2::init().expect("Failed to initialize sdl2");

        let mut canvas = {
            // evaluation block
            let video = sdl.video().expect("Failed to acquire display");

            let window = video
                .window("Tetris", INIT_SIZE.x, INIT_SIZE.y)
                .position_centered()
                .resizable()
                .build()
                .expect("Failed to create window");

            window
                .into_canvas()
                .accelerated()
                .present_vsync()
                .build()
                .expect("Failed to get render canvas")
        };

        let mut events = sdl.event_pump().expect("Failed to get event pump");

        loop {
            for event in events.poll_iter() {
                match dbg!(event) {
                    // log any events with dbg
                    Event::Quit { .. } => return,
                    _ => {}
                }
            }

            draw(&mut canvas, &engine);
        }

        let interface = Self { engine };

        drop(interface);
        todo!("Run the game");
    }
}

fn draw(canvas: &mut Canvas<Window>, engine: &Engine) {
    canvas.set_draw_color(BACKGROUND_COLOR);
    canvas.clear();
    canvas.set_draw_color(Color::WHITE);

    // the square into which we draw and the margin which can be either on the left/right or top/bottom (because the window is resizable)
    let ui_square = {
        let Vector2 { x, y } = Vector2::from(canvas.viewport().size())
            .cast::<i32>()
            .unwrap();

        if x > y {
            // landscape, we have top and bottom black margins
            let midpoint = x / 2;
            let left_edge = midpoint - (y / 2);
            Rect::new(left_edge, 0, y as u32, y as u32)
        } else {
            // portrait, we have left and right black margins
            let midpoint = y / 2;
            let top_edge = midpoint - (x / 2);
            Rect::new(0, top_edge, x as u32, x as u32)
        }
    };
    // canvas.draw_rect(ui_square).unwrap();

    let matrix = {
        // where the tetriminos fly down - the container of it (we will not draw it but use it for layouting)
        let mut middle_section = ui_square.clone();
        middle_section.set_width(middle_section.width() / 2);
        middle_section.center_on(ui_square.center());

        // the actual matrix container
        let mut matrix = middle_section.clone();
        matrix.resize(
            (matrix.width() as f32 * (7.0 / 8.0)) as _, // 7/8ths of full height/width
            (matrix.height() as f32 * (7.0 / 8.0)) as _,
        );
        matrix.center_on(middle_section.center());

        matrix
    };

    // top right container for coming up tetrimino
    let up_next: Rect = {
        // its bounding box
        let mut rect = ui_square.clone();
        let quarter = ui_square.width() / 4;
        rect.resize(
            quarter, // quarter of full width/height
            quarter,
        );
        rect.offset((quarter * 3) as _, 0);

        // 3/4s of the above bounding box
        let inner_dim = rect.width() * 3 / 4;
        let mut inner = rect.clone();
        inner.resize(
            inner_dim as _, // 3/4ths of full height/width
            inner_dim,
        );
        inner.center_on(rect.center());

        inner
    };

    // top left container for hold tetrimino
    let hold: Rect = {
        // its bounding box
        let mut rect = ui_square.clone();
        let quarter = ui_square.width() / 4;
        rect.resize(
            quarter, // quarter of full width/height
            quarter,
        );

        // 3/4s of the above bounding box
        let inner_dim = rect.width() * 3 / 4;
        let mut inner = rect.clone();
        inner.resize(
            inner_dim, // 3/4ths of full height/width
            inner_dim,
        );
        inner.center_on(rect.center());

        inner
    };

    // bottom left where next tetriminos are displayed
    let queue: Rect = {
        // its bounding box
        let mut rect = ui_square.clone();
        let quarter = ui_square.width() / 4;
        rect.resize(
            quarter, // quarter of full width/height
            3 * quarter,
        );
        rect.offset((3 * quarter) as _, quarter as _);

        // 5/8s of the above bounding box
        let inner_width = rect.width() * 5 / 8;
        let inner_height = rect.height() * 23 / 24;
        let mut inner = rect.clone();
        inner.resize(inner_width, inner_height);
        inner.center_on(rect.center());
        inner.set_y(rect.top());

        inner
    };

    // bottom left score box
    let score: Rect = {
        // its bounding box
        let mut rect = ui_square.clone();
        let half = ui_square.width() / 2;
        let quarter = ui_square.width() / 4;
        let sixteenth = half / 8;
        rect.resize(
            quarter, // quarter of full width/height
            2 * quarter,
        );
        rect.offset(0, 5 * sixteenth as i32);

        // 5/8s of the above bounding box
        let mut inner = rect.clone();
        let inner_width = rect.width() * 7 / 8;
        inner.set_width(inner_width);
        inner.center_on(rect.center());
        inner.set_y(rect.top());

        inner
    };

    canvas.set_draw_color(MATRIX_COLOR);
    canvas.fill_rect(matrix).unwrap();
    canvas.fill_rect(up_next).unwrap();
    canvas.fill_rect(hold).unwrap();
    canvas.fill_rect(queue).unwrap();
    canvas.fill_rect(score).unwrap();

    let matrix_origin = matrix.bottom_left();
    let (matrix_width, matrix_height) = matrix.size();
    let matrix_dims = Vector2::from(matrix.size());
    let matrix_cells = Vector2::new(Matrix::WIDTH, Matrix::HEIGHT)
        .cast::<u32>()
        .unwrap();

    for (coord, cell) in engine.cells() {
        let Some(cell_color) = cell else {
            continue;
        };
        // // we get the width from the next cells coordinates because otherwise we end up with a rounding error
        // let this_x = (coord.x as u32 + 0) * matrix_width / Matrix::WIDTH as u32;
        // let this_y = (coord.y as u32 + 1) * matrix_height / Matrix::HEIGHT as u32;

        // let next_x = (coord.x as u32 + 1) * matrix_width / Matrix::WIDTH as u32;
        // let prev_y = (coord.y as u32 + 0) * matrix_height / Matrix::HEIGHT as u32; // we take the previous y because that one will be ABOVE it

        // this is just a more complex version of the thing above which is much easier to understand
        let coord = coord.to_vec().cast::<u32>().unwrap();
        let this = (coord + Vector2::new(0, 1))
            .mul_element_wise(matrix_dims)
            .div_element_wise(matrix_cells);
        let next = (coord + Vector2::new(1, 0))
            .mul_element_wise(matrix_dims)
            .div_element_wise(matrix_cells);

        // our matrix goes bottom left +, their draw matrix goes from top left +, so we need to do some translation
        let cell_rect = Rect::new(
            matrix_origin.x + this.x as i32,
            matrix_origin.y - this.y as i32, // we subtract so we go up instead of down since origin is top left for the draw matrix (we also add one since the rect is drawn in the opposite direction)
            next.x - this.x,                 // next x is "to the right"
            this.y - next.y,                 // prev_y is "higher"
        );

        canvas.set_draw_color(cell_color.screen_color());
        canvas.fill_rect(cell_rect).unwrap();
    }

    canvas.present();
}
