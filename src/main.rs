#![allow(dead_code)]
#![feature(generic_const_exprs, array_chunks, new_range_api)]

use engine::Engine;
use interface::Interface;

mod engine;
mod interface;

fn main() {
    let engine = Engine::new();

    let mut interface = Interface::new(engine);
    drop(interface.run());
}
