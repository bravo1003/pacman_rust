#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Right,
    Up,
    Left,
    Down,
}

impl Direction {
    /// The order in which ghosts consider their options.
    pub const ALL: [Direction; 4] = [Self::Right, Self::Up, Self::Left, Self::Down];

    pub fn delta(self) -> (i32, i32) {
        match self {
            Self::Right => (1, 0),
            Self::Up => (0, -1),
            Self::Left => (-1, 0),
            Self::Down => (0, 1),
        }
    }

    pub fn opposite(self) -> Self {
        match self {
            Self::Right => Self::Left,
            Self::Up => Self::Down,
            Self::Left => Self::Right,
            Self::Down => Self::Up,
        }
    }
}
