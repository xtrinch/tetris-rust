use sdl2::keyboard::Keycode;

use crate::engine::{move_kind::MoveKind, piece_rotation::Rotation};

// types of actions the keyboard can make
pub enum Input {
    Move(MoveKind),
    Rotation(Rotation),
    SoftDrop,
    HardDrop,
    Pause,
    Hold,
    Continue,
}

// map various keyboard keys to actions within the game
impl Input {
    pub fn try_from(key: Keycode, next_rotation: Option<Rotation>) -> Result<Input, ()> {
        println!("{:?}", key);
        Ok(match key {
            Keycode::Right => Self::Move(MoveKind::Right),
            Keycode::Left => Self::Move(MoveKind::Left),
            Keycode::Return => Self::Continue,
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
