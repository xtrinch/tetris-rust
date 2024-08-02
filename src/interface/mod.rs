use cgmath::Vector2;
use sdl2::{event::Event, pixels::Color, rect::Rect};
use std::cmp::min;

use crate::engine::Engine;

pub struct Interface {
    engine: Engine,
}

const INIT_SIZE: Vector2<u32> = Vector2::new(1024, 1024);
const BACKGROUND_COLOR: Color = Color::RGB(0x10, 0x10, 0x18);

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

            canvas.set_draw_color(BACKGROUND_COLOR);
            canvas.clear();

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

            canvas.set_draw_color(Color::WHITE);
            canvas.draw_rect(ui_square).unwrap();
            canvas.present();
        }

        let interface = Self { engine };

        drop(interface);
        todo!("Run the game");
    }
}
