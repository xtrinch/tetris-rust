#![allow(dead_code)]
use engine::Engine;
use interface::Interface;

mod engine;
mod interface;

fn main() {
    println!("Hello, world!");

    let engine = Engine::new();
    Interface::run(engine);
}
