use crate::engine::Engine;
use cancellable_timer::{Canceller, Timer as CancellableTimer};
use cell_draw::CellDrawContext;
use cgmath::Vector2;
use input::Input;
use sdl2::ttf::Sdl2TtfContext;
use sdl2::{event::Event, pixels::Color, rect::Rect, render::Canvas, video::Window};
use sdl2::{EventPump, EventSubsystem, Sdl};
use std::path::Path;
use std::time::Duration;
use sub_rect::{Align, SubRect};
use text_draw::TextDrawContext;

mod cell_draw;
mod input;
mod render_traits;
mod sub_rect;
mod text_draw;

const INIT_SIZE: Vector2<u32> = Vector2::new(1024, 1024);
const BACKGROUND_COLOR: Color = Color::RGB(0x10, 0x10, 0x18);
const MATRIX_COLOR: Color = Color::RGB(0x66, 0x77, 0x77);
const MATRIX_CONTAINER_COLOR: Color = Color::RGB(0x22, 0x22, 0x22);
const PLACEHOLDER_2: Color = Color::RGB(0x66, 0x77, 0x77);
const PLACEHOLDER_3: Color = Color::RGB(0x77, 0x88, 0x88);

// event structs
struct Tick; // basically same as type Tick=()
struct LockdownTick;
struct SoftDropTick;
struct Sleep(Duration);

pub struct Interface {
    pub engine: Engine,
    pub sdl: Sdl,
    pub canvas: Canvas<Window>,
    pub ttf_context: Sdl2TtfContext,
    pub static_event_subsystem: &'static EventSubsystem,
    pub timer_lockdown: Option<Canceller>,
    pub timer_tick: Option<Canceller>,
}

impl Interface {
    pub fn new(engine: Engine) -> Self {
        let sdl: Sdl = sdl2::init().expect("Failed to initialize sdl2");
        let video = sdl.video().expect("Failed to acquire display");
        let canvas = {
            // evaluation block
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

        let static_event_subsystem: &'static _ = Box::leak(Box::new(
            sdl.event().expect("Failed to acquire event subsystem"),
        ));

        Self {
            engine,
            sdl,
            canvas,
            ttf_context,
            static_event_subsystem,
            timer_lockdown: None,
            timer_tick: None,
        }
    }

    pub fn run(&mut self) -> Result<(), String> {
        /*
        A tetrimino that is Hard dropped Locks down immediately. However, if a tetrimino
        naturally falls or Soft drops onto a Surface, it is given 0.5 seconds on a Lock
        down timer before it actually Locks down.
        */

        #[derive(Clone, Copy, PartialEq, Debug)]
        enum State {
            Paused,
            SoftDropping,
            LockingDown,
            LockedDown,
            TickingDown,
        }
        let mut state = State::TickingDown;

        // whether we should redraw or not
        let mut dirty: bool = true;
        // let mut cursor_locked_down: bool = false;
        // let mut paused = false;
        // let mut hold_lock: bool = false;
        // let mut is_soft_drop = false;
        // let mut locking_down = false; // TODO: perhaps best to have a "state" enum instead of relying on this

        self.static_event_subsystem
            .register_custom_event::<Tick>()
            .unwrap();
        self.static_event_subsystem
            .register_custom_event::<LockdownTick>()
            .unwrap();

        self.engine.create_top_cursor(None);

        self.static_event_subsystem.push_custom_event(Tick).unwrap();

        loop {
            for event in self.sdl.event_pump().unwrap().poll_iter() {
                // match dbg!(event) {
                match event {
                    // log any events with dbg
                    Event::Quit { .. } => {
                        return Ok(());
                    }
                    Event::User { .. } if event.as_user_event_type::<Tick>().is_some() => {
                        println!("Timer ticky picky?{:?}", state);
                        if state == State::LockingDown {
                            continue;
                        }

                        self.set_tick_timer(state == State::SoftDropping);

                        if state == State::Paused {
                            continue;
                        };

                        // if we have a cursor to tick down, tick it down :)
                        if self.engine.ticked_down_cursor().is_some() {
                            self.engine.try_tick_down();
                            let has_hit_bottom = self.engine.cursor_has_hit_bottom();

                            if has_hit_bottom {
                                state = State::LockingDown;

                                // add event after 0.5s!
                                self.set_lockdown_timer();
                            }
                        }

                        dirty = true;
                    }
                    Event::User { .. } if event.as_user_event_type::<LockdownTick>().is_some() => {
                        println!("Lockdown ick event? {:?}", state);
                        if state != State::LockingDown {
                            continue;
                        }
                        // the Lock down timer resets to 0.5 seconds if the player simply moves or rotates the tetrimino.
                        self.engine.place_cursor();
                        self.engine.create_top_cursor(None);

                        dirty = true;
                        state = State::LockedDown;

                        self.set_tick_timer(state == State::SoftDropping);
                    }
                    Event::KeyUp {
                        keycode: Some(key), ..
                    } => {
                        if let Ok(input) = Input::try_from(key, self.engine.next_cursor_rotation())
                        {
                            match input {
                                Input::SoftDrop => {
                                    if (state == State::SoftDropping) {
                                        state = State::TickingDown;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    Event::KeyDown {
                        keycode: Some(key), ..
                    } => {
                        if let Ok(input) = Input::try_from(key, self.engine.next_cursor_rotation())
                        {
                            match input {
                                Input::Move(kind) => {
                                    // TODO: to func
                                    if state == State::LockingDown {
                                        self.cancel_set_lockdown_timer();
                                        self.set_lockdown_timer();
                                    }

                                    self.engine.move_cursor(kind);
                                }
                                Input::HardDrop => {
                                    self.engine.hard_drop(); // hard drop
                                    self.engine.create_top_cursor(None);
                                    state = State::LockedDown;
                                }
                                Input::SoftDrop => {
                                    if state != State::SoftDropping && state != State::LockingDown {
                                        state = State::SoftDropping;

                                        self.cancel_set_tick_timer();
                                        self.set_tick_timer(state == State::SoftDropping);
                                    }
                                }
                                Input::Rotation(kind) => {
                                    self.engine.rotate_and_adjust_cursor(kind);
                                }
                                Input::Pause => {
                                    if (state == State::Paused) {
                                        state = State::TickingDown;
                                    } else {
                                        state = State::Paused;
                                    }
                                }
                                Input::Hold => {
                                    self.engine.try_hold();
                                }
                            }
                            dirty = true
                        }
                    }
                    _ => {}
                }
            }

            // scan the board, see what lines need to be cleared
            if state == State::LockedDown {
                self.engine.line_clear(|indices| ());
                state = State::TickingDown;
            }
            if dirty {
                self.draw();
            }
            dirty = false;
        }
    }

    fn cancel_set_tick_timer(&mut self) {
        if self.timer_tick.is_some() {
            self.timer_tick.as_ref().unwrap().cancel();
        }
    }

    fn cancel_set_lockdown_timer(&mut self) {
        if self.timer_lockdown.is_some() {
            self.timer_lockdown.as_ref().unwrap().cancel();
        }
    }

    fn set_tick_timer(&mut self, is_soft_drop: bool) {
        // TODO: to state is soft drop
        let s = self.static_event_subsystem;
        self.timer_tick = Some(
            CancellableTimer::after(
                self.engine.drop_time(is_soft_drop),
                (move |err| {
                    if err.is_err() {
                        return;
                    }
                    s.push_custom_event(Tick).unwrap();
                }),
            )
            .unwrap(),
        )
    }

    fn set_lockdown_timer(&mut self) {
        let s = self.static_event_subsystem;
        self.timer_lockdown = Some(
            CancellableTimer::after(
                Duration::from_millis(500),
                (move |err| {
                    if err.is_err() {
                        return;
                    }
                    s.push_custom_event(LockdownTick).unwrap();
                }),
            )
            .unwrap(),
        )
    }

    fn draw(&mut self) {
        // Load the font
        let path: &Path = Path::new("assets/NewAmsterdam-Regular.ttf");
        let mut font = self
            .ttf_context
            .load_font(path, 512)
            .expect("Failed to load font");

        self.canvas.set_draw_color(BACKGROUND_COLOR);
        self.canvas.clear();
        self.canvas.set_draw_color(Color::WHITE);

        let viewport = self.canvas.viewport();

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

        let matrix_container = ui_square1.sub_rect((0.5, 1.0), None); // half of the width and full height, center alignment by default

        let matrix1 = ui_square1
            .sub_rect((0.5, 1.0), None) // half of the width and full height, center alignment by default
            .sub_rect((7.0 / 8.0, 7.0 / 8.0), None); // 7/8ths of the width and 7/8ths of the height, center by default

        // top right container for coming up tetrimino
        let up_next1 = ui_square1
            .sub_rect((0.25, 0.25), Some((Align::Far, Align::Near))) // top right container
            .sub_rect((7.0 / 8.0, 7.0 / 8.0), Some((Align::Center, Align::Center))); // inside the top right container

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

        self.canvas.set_draw_color(MATRIX_CONTAINER_COLOR);
        self.canvas.fill_rect(Rect::from(matrix_container)).unwrap();

        self.canvas.set_draw_color(MATRIX_COLOR);

        for subrect in [&matrix1, &up_next1, &hold1, &queue1, &score1] {
            self.canvas.fill_rect(Rect::from(subrect)).unwrap();
        }

        let mut cell_draw_ctx: CellDrawContext<
            { Engine::MATRIX_WIDTH },
            { Engine::MATRIX_HEIGHT },
        > = CellDrawContext {
            origin: matrix1.bottom_left(),
            dims: Vector2::from(matrix1.size()),
            canvas: &mut self.canvas,
            matrix: &self.engine.matrix, // TODO: figure our how to pass the iter instead of the whole matrix
        };

        cell_draw_ctx.draw_matrix();

        if let Some((cursor_cells, cursor_color, _)) = self.engine.cursor_info() {
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
            canvas: &mut self.canvas,
            matrix: &self.engine.up_next_matrix,
        };

        up_next_cell_draw_ctx.draw_matrix();

        let mut remaining_next_cell_draw_ctx: CellDrawContext<
            { Engine::REMAINING_NEXT_MATRIX_WIDTH },
            { Engine::REMAINING_NEXT_MATRIX_HEIGHT },
        > = CellDrawContext {
            origin: queue1.bottom_left(),
            dims: Vector2::from(queue1.size()),
            canvas: &mut self.canvas,
            matrix: &self.engine.queue_matrix,
        };

        remaining_next_cell_draw_ctx.draw_matrix();

        let mut hold_cell_draw_ctx: CellDrawContext<
            { Engine::SINGLE_TETRIMINO_MATRIX_WIDTH },
            { Engine::SINGLE_TETRIMINO_MATRIX_HEIGHT },
        > = CellDrawContext {
            origin: hold1.bottom_left(),
            dims: Vector2::from(hold1.size()),
            canvas: &mut self.canvas,
            matrix: &self.engine.hold_matrix,
        };

        hold_cell_draw_ctx.draw_matrix();

        // up next text
        let up_next_text = up_next1.sub_rect((0.5, 0.2), Some((Align::Center, Align::Near)));

        let mut text_draw_ctx: TextDrawContext = TextDrawContext {
            canvas: &mut self.canvas,
            font: &font,
            text: "UP NEXT",
            rect: up_next_text,
        };
        text_draw_ctx.draw_text();

        // hold text
        let hold_text = hold1.sub_rect((0.5, 0.25), Some((Align::Center, Align::Near)));

        let mut text_draw_ctx: TextDrawContext = TextDrawContext {
            canvas: &mut self.canvas,
            font: &font,
            text: "HOLD",
            rect: hold_text,
        };
        text_draw_ctx.draw_text();

        let score_top = score1.sub_rect((1.0, 0.5), Some((Align::Center, Align::Near)));
        let score_bottom = score1.sub_rect((1.0, 0.5), Some((Align::Center, Align::Far)));

        // level text
        let level_text = score_top.sub_rect((0.5, 0.25), Some((Align::Center, Align::Near)));

        let mut text_draw_ctx: TextDrawContext = TextDrawContext {
            canvas: &mut self.canvas,
            font: &font,
            text: "LEVEL",
            rect: level_text,
        };
        text_draw_ctx.draw_text();

        // level text
        let level_text = score_top.sub_rect((0.8, 0.85), Some((Align::Center, Align::Far)));

        let level: u8 = self.engine.level;

        let mut text_draw_ctx: TextDrawContext = TextDrawContext {
            canvas: &mut self.canvas,
            font: &font,
            text: &format!("  {level}  "),
            rect: level_text,
        };
        text_draw_ctx.draw_text();

        // lines text
        let lines_text = score_bottom.sub_rect((0.5, 0.25), Some((Align::Center, Align::Near)));

        let mut text_draw_ctx: TextDrawContext = TextDrawContext {
            canvas: &mut self.canvas,
            font: &font,
            text: "SCORE",
            rect: lines_text,
        };
        text_draw_ctx.draw_text();

        // lines text
        let lines_text = score_bottom.sub_rect((0.8, 0.85), Some((Align::Center, Align::Far)));

        let score = self.engine.score;
        let mut text_draw_ctx: TextDrawContext = TextDrawContext {
            canvas: &mut self.canvas,
            font: &font,
            text: &format!("  {score}  "),
            rect: lines_text,
        };
        text_draw_ctx.draw_text();

        self.canvas.present();
    }
}
