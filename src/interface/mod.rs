use crate::engine::color::TetriminoColor;
use crate::engine::matrix::Matrix;
use crate::engine::move_kind::MoveKind;
use crate::engine::piece::Rotation;
use crate::engine::{Coordinate, Engine};
use crate::interface::render_traits::ScreenColor;
use cgmath::{ElementWise, EuclideanSpace, Point2, Vector2};
use sdl2::keyboard::Keycode;
use sdl2::{event::Event, pixels::Color, rect::Rect, render::Canvas, video::Window};
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

    pub fn run(mut engine: Engine) {
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

        let mut events = sdl.event_pump().expect("Failed to get event pump");

        event_subsystem.push_custom_event(Tick).unwrap();

        // whether we should redraw or not
        let mut dirty: bool = true;
        let mut timer;
        let mut lock_down: bool = false;
        let mut paused = false;

        engine.create_top_cursor();

        loop {
            for event in events.poll_iter() {
                // match dbg!(event) {
                match event {
                    // log any events with dbg
                    Event::Quit { .. } => return,
                    Event::User { .. } if event.as_user_event_type::<Tick>().is_some() => {
                        timer = timer_subsystem.add_timer(
                            engine.drop_time().as_millis() as _,
                            Box::new(|| {
                                println!("Tick event timer triggered");
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
                        engine.create_top_cursor();

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
                                    engine.create_top_cursor();
                                    lock_down = true
                                }
                                Input::SoftDrop => println!("Soft drop tick"),
                                Input::Rotation(kind) => engine.rotate_cursor(kind),
                                Input::Pause => {
                                    paused = !paused;
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
            }
            if dirty {
                draw(&mut canvas, &engine);
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
            _ => return Err(()),
        })
    }
}

fn draw(canvas: &mut Canvas<Window>, engine: &Engine) {
    canvas.set_draw_color(BACKGROUND_COLOR);
    canvas.clear();
    canvas.set_draw_color(Color::WHITE);

    let viewport = canvas.viewport();

    // the design is all based upon a 16x15 grid which is further divided into 4ths (see grid.png) -
    // the system is based upon first positioning the container, then an inner rect relative to id

    // the square into which we draw and the margin which can be either on the left/right or top/bottom (because the window is resizable)
    let ui_square = {
        let Vector2 { x, y } = Vector2::from(viewport.size()).cast::<i32>().unwrap();

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
        .sub_rect((0.75, 0.75), None);

    // bottom left where next tetriminos are displayed
    let queue1 = ui_square1
        .sub_rect((0.25, 0.75), Some((Align::Far, Align::Far)))
        .sub_rect((5.0 / 8.0, 23.0 / 24.0), Some((Align::Center, Align::Near)));

    // bottom left score box
    let score1 = ui_square1
        .sub_rect((0.25, 11.0 / 16.0), Some((Align::Near, Align::Far)))
        .sub_rect((7.0 / 8.0, 8.0 / 11.0), Some((Align::Center, Align::Near)));

    canvas.set_draw_color(MATRIX_COLOR);

    for subrect in [&matrix1, &up_next1, &hold1, &queue1, &score1] {
        canvas.fill_rect(Rect::from(subrect)).unwrap();
    }

    let mut cell_draw_ctx = CellDrawContext {
        origin: matrix1.bottom_left(),
        dims: Vector2::from(matrix1.size()),
        canvas,
    };

    for (coord, cell) in engine.cells() {
        cell_draw_ctx.draw_border(coord);
    }

    for (coord, cell) in engine.cells() {
        cell_draw_ctx.try_draw_cell(coord, cell);
    }

    if let Some((cursor_cells, cursor_color, _)) = engine.cursor_info() {
        for coord in cursor_cells {
            cell_draw_ctx.try_draw_cell(coord, Some(cursor_color));
        }
    }

    canvas.present();
}

// we need a lifetime because we have a mutable reference
struct CellDrawContext<'canvas> {
    origin: Point2<i32>,
    dims: Vector2<u32>,
    canvas: &'canvas mut Canvas<Window>,
}

impl CellDrawContext<'_> {
    const CELL_COUNT: Vector2<u32> = Vector2::new(Matrix::WIDTH as u32, Matrix::HEIGHT as u32);

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
