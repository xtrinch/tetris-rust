#![allow(dead_code)]
#![feature(generic_const_exprs, array_chunks, new_range_api)]

use engine::Engine;
use interface::Interface;

mod engine;
mod interface;

fn main() {
    println!("Hello, world!");

    // // line across the bottom that leaves three spaces
    // for col in 0..7 {
    //     matrix[(col, 0).into()] = Some(Color::Green);
    // }

    let engine = Engine::new();

    let mut interface = Interface::new(engine);
    interface.run();
}
