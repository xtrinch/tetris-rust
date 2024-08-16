use crate::interface::render_traits::ScreenColor;
use cgmath::ElementWise;
use cgmath::EuclideanSpace;
use cgmath::{Point2, Vector2};
use sdl2::render::TextureQuery;
use sdl2::ttf::Font;
use sdl2::{pixels::Color, rect::Rect, render::Canvas, video::Window};

use super::sub_rect::Align;
use super::sub_rect::SubRect;

// we need a lifetime because we have a mutable reference
pub struct TextDrawContext<'canvas, 'canvas1> {
    pub font: &'canvas Font<'canvas, 'canvas1>,
    pub canvas: &'canvas mut Canvas<Window>,
    pub text: &'canvas str,
    pub rect: SubRect,
}

impl TextDrawContext<'_, '_> {
    pub fn draw_text(&mut self) {
        let texture_creator = self.canvas.texture_creator();

        // render a surface, and convert it to a texture bound to the canvas
        let surface = self
            .font
            .render(self.text)
            .blended(Color::WHITE)
            .map_err(|e| e.to_string())
            .expect("Failed to create surface");
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())
            .expect("Failed to create texture");

        let TextureQuery { width, height, .. } = texture.query();

        let container = SubRect::absolute(
            Rect::from(self.rect),
            ((width / 512) as f32, (height / 512) as f32),
            None,
        );

        self.canvas
            .copy(&texture, None, Some(Rect::from(container)))
            .expect("Failed to copy to canvas");
    }
}
