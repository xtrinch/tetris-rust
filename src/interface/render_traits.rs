use sdl2::pixels::Color as SdlColor;

use crate::engine::color::TetriminoColor;

pub trait ScreenColor {
    fn screen_color(&self) -> SdlColor;
}

// we pull it out rather than putting it directly on the semantic color so this is a member of the interface and NOT the engine
impl ScreenColor for TetriminoColor {
    fn screen_color(&self) -> SdlColor {
        match self {
            TetriminoColor::Yellow => SdlColor::RGB(0xed, 0xd4, 0x00),
            TetriminoColor::Cyan => SdlColor::RGB(0x72, 0x9f, 0xcf),
            TetriminoColor::Purple => SdlColor::RGB(0x75, 0x50, 0x7b),
            TetriminoColor::Orange => SdlColor::RGB(0xf5, 0x79, 0x00),
            TetriminoColor::Blue => SdlColor::RGB(0x34, 0x65, 0xa4),
            TetriminoColor::Green => SdlColor::RGB(0x73, 0xd2, 0x16),
            TetriminoColor::Red => SdlColor::RGB(0xef, 0x29, 0x29),
        }
    }
}
