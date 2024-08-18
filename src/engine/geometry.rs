use super::Coordinate;

// add a trait so we can grid increment a position on the matrix
pub trait GridIncrement<const WIDTH: usize>: Sized {
    type Width;

    // because we're taking size by value we need to know the size (so we :Sized)
    fn grid_incd(mut self) -> Self {
        self.grid_inc();
        self
    }

    fn grid_inc(&mut self);
}

impl<const WIDTH: usize> GridIncrement<WIDTH> for Coordinate {
    type Width = usize;

    fn grid_inc(&mut self) {
        self.x += 1;
        self.x %= WIDTH;
        if self.x == 0 {
            self.y += 1;
        }
    }
}
