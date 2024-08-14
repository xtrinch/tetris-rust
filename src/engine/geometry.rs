use super::{matrix::Matrix, Coordinate};

// add a trait so we can grid increment a position on the matrix
pub trait GridIncrement: Sized {
    type Width;
    const WIDTH: Self::Width;

    // because we're taking size by value we need to know the size (so we :Sized)
    fn grid_incd(mut self) -> Self {
        self.grid_inc();
        self
    }

    fn grid_inc(&mut self);
}

impl GridIncrement for Coordinate {
    type Width = usize;
    const WIDTH: Self::Width = Matrix::WIDTH;

    fn grid_inc(&mut self) {
        self.x += 1;
        self.x %= Matrix::WIDTH;
        if self.x == 0 {
            self.y += 1;
        }
    }
}
