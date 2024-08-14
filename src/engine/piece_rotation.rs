use super::Offset;
use cgmath::Zero;

#[derive(Clone, Copy, PartialEq, Debug)]
// how the piece is rotated
pub enum Rotation {
    N,
    E,
    S,
    W,
}

impl Rotation {
    pub fn intrinsic_offset(&self) -> Offset {
        // this we need to then multiply by grid size
        match self {
            Self::N => Offset::zero(),
            Self::E => Offset::new(0, 1), // 2nd quadrant, so y has moved
            Self::S => Offset::new(1, 1), // 3rd quadrant, so both x and y have moved down
            Self::W => Offset::new(1, 0), // 4th quadrant, so only x has moved
        }
    }

    pub fn next_rotation(&self) -> Self {
        match self {
            Self::N => Self::E,
            Self::E => Self::S,
            Self::S => Self::W,
            Self::W => Self::N,
        }
    }
}

// multiply vector by a rotation -> for rotating relative coordinates of a piece
impl std::ops::Mul<Rotation> for Offset {
    type Output = Self;

    fn mul(self, rotation: Rotation) -> Self::Output {
        match rotation {
            Rotation::N => self, // no op as the coordinates are already north facing
            Rotation::S => Self::new(-self.x, -self.y), // flip x & y axis
            Rotation::E => Self::new(self.y, -self.x),
            Rotation::W => Self::new(-self.y, self.x),
        }
    }
}
