#![allow(dead_code)]
#![feature(is_sorted, array_chunks, new_range_api)]

use engine::{Color, Engine, Matrix};
use interface::Interface;

mod engine;
mod interface;

fn main() {
    println!("Hello, world!");

    let mut matrix = Matrix::blank();

    // // line across the bottom that leaves three spaces
    // for col in 0..7 {
    //     matrix[(col, 0).into()] = Some(Color::Green);
    // }

    let mut engine = Engine::with_matrix(matrix);

    Interface::run(engine);
}
