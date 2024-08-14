use super::Offset;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum MoveKind {
    Left,
    Right,
}

impl MoveKind {
    pub fn offset(&self) -> Offset {
        match self {
            MoveKind::Left => Offset::new(-1, 0),
            MoveKind::Right => Offset::new(1, 0),
        }
    }
}
