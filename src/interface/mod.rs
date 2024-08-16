use crate::engine::Engine;
use cancellable_timer::Timer as CancellableTimer;
use cell_draw::CellDrawContext;
use cgmath::Vector2;
use input::Input;
use sdl2::timer::Timer;
use sdl2::ttf::Font;
use sdl2::{event::Event, pixels::Color, rect::Rect, render::Canvas, video::Window};
use std::path::Path;
use std::time::Duration;
use sub_rect::{Align, SubRect};
use text_draw::TextDrawContext;

mod cell_draw;
mod input;
mod render_traits;
mod sub_rect;
mod text_draw;

pub struct Interface {
    engine: Engine,
}

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
        let path: &Path = Path::new("assets/NewAmsterdam-Regular.ttf");
        let mut font = ttf_context
            .load_font(path, 512)
            .expect("Failed to load font");

        let mut events = sdl.event_pump().expect("Failed to get event pump");

        event_subsystem.push_custom_event(Tick).unwrap();

        /*
        A tetrimino that is Hard dropped Locks down immediately. However, if a tetrimino
        naturally falls or Soft drops onto a Surface, it is given 0.5 seconds on a Lock
        down timer before it actually Locks down.
        */

        // whether we should redraw or not
        let mut dirty: bool = true;
        let mut timer_tick: Timer;
        let mut timer_lockdown: Timer;
        let mut lock_down: bool = false;
        let mut paused = false;
        let mut hold_lock: bool = false;
        let mut is_soft_drop = false;

        engine.create_top_cursor(None);

        let tim = CancellableTimer::after(Duration::from_secs(3), |tets| {
            print!("{:?}", tets);
            if tets.is_err() {
                return;
            }
            println!("OI!");
        })
        .unwrap();
        tim.cancel();

        loop {
            for event in events.poll_iter() {
                // match dbg!(event) {
                match event {
                    // log any events with dbg
                    Event::Quit { .. } => return Ok(()),
                    Event::User { .. } if event.as_user_event_type::<Tick>().is_some() => {
                        println!("{}", is_soft_drop);
                        timer_tick = timer_subsystem.add_timer(
                            engine.drop_time(is_soft_drop).as_millis() as _,
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
                                // add event after 0.5s!
                                timer_lockdown = timer_subsystem.add_timer(
                                    Duration::from_millis(500).as_millis() as _,
                                    Box::new(|| {
                                        event_subsystem.push_custom_event(LockdownTick).unwrap();
                                        0
                                    }),
                                );
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
                    Event::KeyUp {
                        keycode: Some(key), ..
                    } => {
                        if let Ok(input) = Input::try_from(key, engine.next_cursor_rotation()) {
                            match input {
                                Input::SoftDrop => {
                                    println!("Soft drop tick up");
                                    is_soft_drop = false;
                                }
                                _ => {}
                            }
                        }
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
                                Input::SoftDrop => {
                                    println!("Soft drop tick");
                                    if !is_soft_drop {
                                        is_soft_drop = true;
                                        // TODO: to func?
                                        timer_tick = timer_subsystem.add_timer(
                                            engine.drop_time(is_soft_drop).as_millis() as _,
                                            Box::new(|| {
                                                event_subsystem.push_custom_event(Tick).unwrap();
                                                0
                                            }),
                                        );
                                    }
                                }
                                Input::Rotation(kind) => {
                                    engine.rotate_and_adjust_cursor(kind);
                                }
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
                is_soft_drop = false;
            }
            if dirty {
                draw(&mut canvas, &mut font, &engine);
            }
            dirty = false;
        }
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

    canvas.set_draw_color(MATRIX_CONTAINER_COLOR);
    canvas.fill_rect(Rect::from(matrix_container)).unwrap();

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

    // up next text
    let up_next_text = up_next1.sub_rect((0.5, 0.2), Some((Align::Center, Align::Near)));

    let mut text_draw_ctx: TextDrawContext = TextDrawContext {
        canvas,
        font,
        text: "UP NEXT",
        rect: up_next_text,
    };
    text_draw_ctx.draw_text();

    // hold text
    let hold_text = hold1.sub_rect((0.5, 0.25), Some((Align::Center, Align::Near)));

    let mut text_draw_ctx: TextDrawContext = TextDrawContext {
        canvas,
        font,
        text: "HOLD",
        rect: hold_text,
    };
    text_draw_ctx.draw_text();

    let score_top = score1.sub_rect((1.0, 0.5), Some((Align::Center, Align::Near)));
    let score_bottom = score1.sub_rect((1.0, 0.5), Some((Align::Center, Align::Far)));

    // level text
    let level_text = score_top.sub_rect((0.5, 0.25), Some((Align::Center, Align::Near)));

    let mut text_draw_ctx: TextDrawContext = TextDrawContext {
        canvas,
        font,
        text: "LEVEL",
        rect: level_text,
    };
    text_draw_ctx.draw_text();

    // level text
    let level_text = score_top.sub_rect((0.8, 0.85), Some((Align::Center, Align::Far)));

    let level: u8 = engine.level;

    let mut text_draw_ctx: TextDrawContext = TextDrawContext {
        canvas,
        font,
        text: &format!("  {level}  "),
        rect: level_text,
    };
    text_draw_ctx.draw_text();

    // lines text
    let lines_text = score_bottom.sub_rect((0.5, 0.25), Some((Align::Center, Align::Near)));

    let mut text_draw_ctx: TextDrawContext = TextDrawContext {
        canvas,
        font,
        text: "SCORE",
        rect: lines_text,
    };
    text_draw_ctx.draw_text();

    // lines text
    let lines_text = score_bottom.sub_rect((0.8, 0.85), Some((Align::Center, Align::Far)));

    let score = engine.score;
    let mut text_draw_ctx: TextDrawContext = TextDrawContext {
        canvas,
        font,
        text: &format!("  {score}  "),
        rect: lines_text,
    };
    text_draw_ctx.draw_text();

    canvas.present();
}
