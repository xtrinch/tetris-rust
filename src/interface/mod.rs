use crate::engine::color::TetriminoColor;
use crate::engine::matrix::{CellIter, Matrix};
use crate::engine::move_kind::MoveKind;
use crate::engine::piece_rotation::Rotation;
use crate::engine::{Coordinate, Engine};
use crate::interface::render_traits::ScreenColor;
use cgmath::{ElementWise, EuclideanSpace, Point2, Vector2};
use sdl2::keyboard::Keycode;
use sdl2::render::TextureQuery;
use sdl2::ttf::Font;
use sdl2::{event::Event, pixels::Color, rect::Rect, render::Canvas, video::Window};
use std::cell;
use std::path::Path;
use std::time::Duration;
use sub_rect::{Align, SubRect};

mod render_traits;
mod sub_rect;

pub struct Interface {
    engine: Engine,
}

const INIT_SIZE: Vector2<u32> = Vector2::new(1024, 1024);
const BACKGROUND_COLOR: Color = Color::RGB(0x10, 0x10, 0x18);
const MATRIX_COLOR: Color = Color::RGB(0x66, 0x77, 0x77);
const PLACEHOLDER_2: Color = Color::RGB(0x66, 0x77, 0x77);
const PLACEHOLDER_3: Color = Color::RGB(0x77, 0x88, 0x88);

// event structs
struct Tick; // basically same as type Tick=()
struct LockdownTick;
struct SoftDropTick;
struct Sleep(Duration);

// handle the annoying Rect i32
macro_rules! rect(
    ($x:expr, $y:expr, $w:expr, $h:expr) => (
        Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
    )
);

impl Interface {
    // fn flush_and_readd_events() {
    //     // flush and readd events
    //     event_subsystem.flush_event(EventType::User);
    //     timer = timer_subsystem.add_timer(
    //         engine.drop_time().as_millis() as _,
    //         Box::new(|| {
    //             println!("Tick event timer triggered");
    //             event_subsystem.push_custom_event(Tick).unwrap();
    //             0
    //         }),
    //     );
    // }

    // Scale fonts to a reasonable size when they're too big (though they might look less smooth)
    fn get_centered_rect(
        rect_width: u32,
        rect_height: u32,
        cons_width: u32,
        cons_height: u32,
    ) -> Rect {
        let wr = rect_width as f32 / cons_width as f32;
        let hr = rect_height as f32 / cons_height as f32;

        let (w, h) = if wr > 1f32 || hr > 1f32 {
            if wr > hr {
                println!("Scaling down! The text will look worse!");
                let h = (rect_height as f32 / wr) as i32;
                (cons_width as i32, h)
            } else {
                println!("Scaling down! The text will look worse!");
                let w = (rect_width as f32 / hr) as i32;
                (w, cons_height as i32)
            }
        } else {
            (rect_width as i32, rect_height as i32)
        };

        let cx = (800 as i32 - w) / 2;
        let cy = (800 as i32 - h) / 2;
        rect!(cx, cy, w, h)
    }

    pub fn run(mut engine: Engine) -> Result<(), String> {
        let sdl = sdl2::init().expect("Failed to initialize sdl2");

        let event_subsystem = sdl.event().expect("Failed to acquire event subsystem");
        event_subsystem.register_custom_event::<Tick>().unwrap();
        event_subsystem
            .register_custom_event::<LockdownTick>()
            .unwrap();

        let timer_subsystem = sdl.timer().expect("Failed to acquire timer subsystem");

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

        let ttf_context = sdl2::ttf::init()
            .map_err(|e| e.to_string())
            .expect("Failed to initialize ttf context");

        // Load the font
        let path: &Path = Path::new("assets/Tinos-Regular.ttf");
        let mut font = ttf_context
            .load_font(path, 128)
            .expect("Failed to load font");

        let mut events = sdl.event_pump().expect("Failed to get event pump");

        event_subsystem.push_custom_event(Tick).unwrap();

        // whether we should redraw or not
        let mut dirty: bool = true;
        let mut timer;
        let mut lock_down: bool = false;
        let mut paused = false;
        let mut hold_lock: bool = false;

        engine.create_top_cursor(None);

        loop {
            for event in events.poll_iter() {
                // match dbg!(event) {
                match event {
                    // log any events with dbg
                    Event::Quit { .. } => return Ok(()),
                    Event::User { .. } if event.as_user_event_type::<Tick>().is_some() => {
                        timer = timer_subsystem.add_timer(
                            engine.drop_time().as_millis() as _,
                            Box::new(|| {
                                event_subsystem.push_custom_event(Tick).unwrap();
                                0
                            }),
                        );

                        if paused {
                            break;
                        };

                        // if we have a cursor to tick down, tick it down :)
                        if engine.ticked_down_cursor().is_some() {
                            engine.try_tick_down();
                            let has_hit_bottom = engine.cursor_has_hit_bottom();

                            if has_hit_bottom {
                                event_subsystem.push_custom_event(LockdownTick).unwrap();
                            }
                        }

                        dirty = true;
                    }
                    Event::User { .. } if event.as_user_event_type::<LockdownTick>().is_some() => {
                        println!("Found lockdown tick event");
                        // the Lock down timer resets to 0.5 seconds if the player simply moves or rotates the tetrimino.
                        engine.place_cursor();
                        engine.create_top_cursor(None);

                        dirty = true;
                        lock_down = true
                    }
                    Event::KeyDown {
                        keycode: Some(key), ..
                    } => {
                        if let Ok(input) = Input::try_from(key, engine.next_cursor_rotation()) {
                            // TODO: flush and readd events if we're in lockdown phase?

                            match input {
                                Input::Move(kind) => drop(engine.move_cursor(kind)),
                                Input::HardDrop => {
                                    engine.hard_drop(); // hard drop
                                    engine.create_top_cursor(None);
                                    lock_down = true
                                }
                                Input::SoftDrop => println!("Soft drop tick"),
                                Input::Rotation(kind) => engine.rotate_cursor(kind),
                                Input::Pause => {
                                    paused = !paused;
                                }
                                Input::Hold => {
                                    if !hold_lock {
                                        engine.try_hold();
                                    }
                                    hold_lock = true;
                                }
                            }
                            dirty = true
                        }
                    }
                    _ => {}
                }
            }

            // scan the board, see what lines need to be cleared
            if lock_down {
                engine.line_clear(|indices| ());
                hold_lock = false;
                lock_down = false;
            }
            if dirty {
                draw(&mut canvas, &mut font, &engine);
            }
            dirty = false;
        }
    }
}

// types of actions the keyboard can make
enum Input {
    Move(MoveKind),
    Rotation(Rotation),
    SoftDrop,
    HardDrop,
    Pause,
    Hold,
}

// map various keyboard keys to actions within the game
impl Input {
    fn try_from(key: Keycode, next_rotation: Option<Rotation>) -> Result<Input, ()> {
        Ok(match key {
            Keycode::Right => Self::Move(MoveKind::Right),
            Keycode::Left => Self::Move(MoveKind::Left),
            Keycode::Up => {
                if let Some(rotation) = next_rotation {
                    Self::Rotation(rotation)
                } else {
                    Self::Rotation(Rotation::N)
                }
            }
            Keycode::Down => Self::SoftDrop,
            Keycode::Space => Self::HardDrop,
            Keycode::NUM_1 => Self::Pause,
            Keycode::C => Self::Hold,
            _ => return Err(()),
        })
    }
}

fn draw(canvas: &mut Canvas<Window>, font: &mut Font, engine: &Engine) {
    canvas.set_draw_color(BACKGROUND_COLOR);
    canvas.clear();
    canvas.set_draw_color(Color::WHITE);

    let viewport = canvas.viewport();

    // the design is all based upon a 16x15 grid which is further divided into 4ths (see grid.png) -
    // the system is based upon first positioning the container, then an inner rect relative to id

    // the square into which we draw and the margin which can be either on the left/right or top/bottom (because the window is resizable)
    // let ui_square = {
    //     let Vector2 { x, y } = Vector2::from(viewport.size()).cast::<i32>().unwrap();

    //     if x > y {
    //         // landscape, we have top and bottom black margins
    //         let midpoint = x / 2;
    //         let left_edge = midpoint - (y / 2);
    //         Rect::new(left_edge, 0, y as u32, y as u32)
    //     } else {
    //         // portrait, we have left and right black margins
    //         let midpoint = y / 2;
    //         let top_edge = midpoint - (x / 2);
    //         Rect::new(0, top_edge, x as u32, x as u32)
    //     }
    // };
    // canvas.draw_rect(ui_square).unwrap();

    // the square into which we draw and the margin which can be either on the left/right or top/bottom (because the window is resizable)
    let ui_square1 = SubRect::absolute(viewport, (1.0, 1.0), None);
    // canvas.draw_rect(Rect::from(ui_square1)).unwrap();

    let matrix1 = ui_square1
        .sub_rect((0.5, 1.0), None) // half of the width and full height, center alignment by default
        .sub_rect((7.0 / 8.0, 7.0 / 8.0), None); // 7/8ths of the width and 7/8ths of the height, center by default

    // top right container for coming up tetrimino
    let up_next1 = ui_square1
        .sub_rect((0.25, 0.25), Some((Align::Far, Align::Near))) // top right container
        .sub_rect((7.0 / 8.0, 7.0 / 8.0), None); // inside the top right container

    // top left container for hold tetrimino
    let hold1 = ui_square1
        .sub_rect((0.25, 0.25), Some((Align::Near, Align::Near)))
        .sub_rect((0.64, 0.64), None);

    // bottom right where next tetriminos are displayed
    let queue1 = ui_square1
        .sub_rect((0.25, 0.75), Some((Align::Far, Align::Far)))
        .sub_rect(
            (5.0 / 10.0, 23.0 / 24.0),
            Some((Align::Center, Align::Near)),
        );

    // bottom left score box
    let score1 = ui_square1
        .sub_rect((0.25, 11.0 / 16.0), Some((Align::Near, Align::Far)))
        .sub_rect((7.0 / 8.0, 8.0 / 11.0), Some((Align::Center, Align::Near)));

    canvas.set_draw_color(MATRIX_COLOR);

    for subrect in [&matrix1, &up_next1, &hold1, &queue1, &score1] {
        canvas.fill_rect(Rect::from(subrect)).unwrap();
    }

    let mut cell_draw_ctx: CellDrawContext<{ Engine::MATRIX_WIDTH }, { Engine::MATRIX_HEIGHT }> =
        CellDrawContext {
            origin: matrix1.bottom_left(),
            dims: Vector2::from(matrix1.size()),
            canvas,
            matrix: &engine.matrix, // TODO: figure our how to pass the iter instead of the whole matrix
        };

    cell_draw_ctx.draw_matrix();

    if let Some((cursor_cells, cursor_color, _)) = engine.cursor_info() {
        for coord in cursor_cells {
            cell_draw_ctx.try_draw_cell(coord, Some(cursor_color));
        }
    }

    let mut up_next_cell_draw_ctx: CellDrawContext<
        { Engine::SINGLE_TETRIMINO_MATRIX_WIDTH },
        { Engine::SINGLE_TETRIMINO_MATRIX_HEIGHT },
    > = CellDrawContext {
        origin: up_next1.bottom_left(),
        dims: Vector2::from(up_next1.size()),
        canvas,
        matrix: &engine.up_next_matrix,
    };

    up_next_cell_draw_ctx.draw_matrix();

    let mut remaining_next_cell_draw_ctx: CellDrawContext<
        { Engine::REMAINING_NEXT_MATRIX_WIDTH },
        { Engine::REMAINING_NEXT_MATRIX_HEIGHT },
    > = CellDrawContext {
        origin: queue1.bottom_left(),
        dims: Vector2::from(queue1.size()),
        canvas,
        matrix: &engine.queue_matrix,
    };

    remaining_next_cell_draw_ctx.draw_matrix();

    let mut hold_cell_draw_ctx: CellDrawContext<
        { Engine::SINGLE_TETRIMINO_MATRIX_WIDTH },
        { Engine::SINGLE_TETRIMINO_MATRIX_HEIGHT },
    > = CellDrawContext {
        origin: hold1.bottom_left(),
        dims: Vector2::from(hold1.size()),
        canvas,
        matrix: &engine.hold_matrix,
    };

    hold_cell_draw_ctx.draw_matrix();

    let texture_creator = canvas.texture_creator();

    // render a surface, and convert it to a texture bound to the canvas
    let surface = font
        .render("Hello Rust!")
        .blended(Color::WHITE)
        .map_err(|e| e.to_string())
        .expect("Failed to create surface");
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())
        .expect("Failed to create texture");

    let TextureQuery { width, height, .. } = texture.query();

    canvas
        .copy(&texture, None, Some(Rect::from(score1)))
        .expect("Failed to copy to canvas");

    canvas.present();
}

// we need a lifetime because we have a mutable reference
struct CellDrawContext<'canvas, const WIDTH: usize, const HEIGHT: usize>
where
    [usize; WIDTH * HEIGHT]:,
{
    origin: Point2<i32>,
    dims: Vector2<u32>,
    canvas: &'canvas mut Canvas<Window>,
    matrix: &'canvas Matrix<WIDTH, HEIGHT>,
}

impl<const WIDTH: usize, const HEIGHT: usize> CellDrawContext<'_, { WIDTH }, { HEIGHT }>
where
    [usize; WIDTH * HEIGHT]:,
{
    const CELL_COUNT: Vector2<u32> = Vector2::new(WIDTH as u32, HEIGHT as u32);

    fn draw_matrix(&mut self) {
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
        let cell_rect = Rect::new(
            self.origin.x + this.x as i32,
            self.origin.y - this.y as i32 - 1, // we subtract so we go up instead of down since origin is top left for the draw matrix (we also add one since the rect is drawn in the opposite direction); -1 is because we do border overlap adjustments
            next.x - this.x + 1, // next x is "to the right", -1 to make the borders overlap
            this.y - next.y + 1, // prev_y is "higher", -1 to make the borders overlap
        );

        cell_rect
    }

    fn try_draw_cell(&mut self, coord: Coordinate, cell: Option<TetriminoColor>) {
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
