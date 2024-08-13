// the design is all based upon a 16x15 grid which is further divided into 4ths (see grid.png) -
// the system is based upon first positioning the container, then an inner rect relative to id

use cgmath::{ElementWise, Point2, Vector2};
use sdl2::rect::Rect;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SubRect {
    outer: Rect,           // stl2 rect which represents the outer area
    ratio: Vector2<f32>, // a two dimensional vector that represents the ratio that we're taking up; 1 if whole area, 0.5 if half of it
    align: Vector2<Align>, // a vertical and a horizontal alignment, where it's going to push the rectangle inside the outer rect
}

impl SubRect {
    // constructing
    pub fn of(outer: Rect, ratio: (f32, f32), align: Option<(Align, Align)>) -> Self {
        Self {
            outer,
            ratio: ratio.into(),
            align: align.unwrap_or((Align::Center, Align::Center)).into(),
        }
    }

    // creates a sub rect of the sub rect
    pub fn sub_rect(&self, ratio: (f32, f32), align: Option<(Align, Align)>) -> Self {
        // parent converted into a rect
        Self::of(Rect::from(self), ratio, align)
    }

    // instead of relative ratio to the parent this will be an absolute ratio to the parent (example is the ui_square which has a ratio of 1:1 inside the draw square)
    pub fn absolute(outer: Rect, ratio: (f32, f32), align: Option<(Align, Align)>) -> Self {
        let Vector2 { x, y } = Vector2::from(outer.size()).cast::<f32>().unwrap();

        // we find the aspect ratio that it needs to be and then squishes it in; e.g. if x is larger, we'd like to make the x smaller to be equal to y for 1:1
        let aspect_correction = Vector2::from(if x > y { (y / x, 1.0) } else { (1.0, x / y) });

        let ratio = Vector2::from(ratio)
            .mul_element_wise(aspect_correction)
            .into();

        Self::of(outer, ratio, align) // we now have relative coords and can create the standard rect
    }

    // total margin horizontally & vertically for the rect
    fn total_margin(&self) -> Vector2<f32> {
        Vector2::from(self.outer.size())
            .cast()
            .unwrap()
            .mul_element_wise(Vector2::new(1.0, 1.0) - self.ratio)
    }

    // get coords of top left for the subrect
    pub fn top_left(&self) -> Point2<i32> {
        let outer_top_left: (i32, i32) = self.outer.top_left().into(); // parent top left
        let margin = self
            .total_margin()
            .mul_element_wise(self.align.map(Align::front_margin));

        Point2::from(outer_top_left) + margin.cast().unwrap()
    }

    // get coords of bottom left for the subrect
    pub fn bottom_left(&self) -> Point2<i32> {
        let outer_bottom_left: (i32, i32) = self.outer.bottom_left().into();
        let margin = self
            .total_margin()
            .mul_element_wise(self.align.map(Align::back_margin))
            .mul_element_wise(Vector2::new(1.0, -1.0));

        Point2::from(outer_bottom_left) + margin.cast().unwrap()
    }

    // size of the subrect
    pub fn size(&self) -> Vector2<u32> {
        let outer_size = Vector2::from(self.outer.size()).cast::<f32>().unwrap(); // parent size

        outer_size
            .mul_element_wise(self.ratio)
            .map(f32::trunc)
            .cast()
            .unwrap()
    }
}

impl From<SubRect> for Rect {
    fn from(region: SubRect) -> Self {
        Rect::from(&region)
    }
}

// get the actual x, y and width
impl From<&SubRect> for Rect {
    fn from(region: &SubRect) -> Self {
        let Point2 { x, y } = region.top_left();
        let Vector2 {
            x: width,
            y: height,
        } = region.size();

        Rect::new(x, y, width, height)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Align {
    Near, // left or top
    Center,
    Far, // right or bottom
}

impl Align {
    pub fn front_margin(self) -> f32 {
        match self {
            Align::Near => 0.0,   // multiply the margin by 0 to get top left
            Align::Center => 0.5, // multiply the margin by 0.5 to get top left
            Align::Far => 1.0,    // multiply the total margin by 1 to get top left
        }
    }

    pub fn back_margin(self) -> f32 {
        1.0 - self.front_margin()
    }
}
