#![allow(dead_code)]
use engine::{Color, Engine, Matrix};
use interface::Interface;

mod engine;
mod interface;

fn main() {
    println!("Hello, world!");

    let mut matrix = Matrix::blank();
    matrix[(1, 1).into()] = Some(Color::Green);

    let engine = Engine::with_matrix(matrix);
    Interface::run(engine);
}
