use crate::direction::Direction;

pub const BOARD_WIDTH: usize = 28;
pub const BOARD_HEIGHT: usize = 36;
/// Edge of a maze tile, in pixels. Actors occupy a tile-sized box.
pub const TILE: i32 = 24;
/// Edge of an actor sprite frame, in pixels.
pub const SPRITE: i32 = 32;
pub const WINDOW_WIDTH: i32 = BOARD_WIDTH as i32 * TILE;
pub const WINDOW_HEIGHT: i32 = BOARD_HEIGHT as i32 * TILE;

/// A pixel position; for actors it is the top-left corner of their box.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn offset(self, direction: Direction, distance: i32) -> Self {
        let (dx, dy) = direction.delta();
        Self::new(self.x + dx * distance, self.y + dy * distance)
    }

    pub fn step(self, direction: Direction) -> Self {
        self.offset(direction, 1)
    }

    /// Teleports through the side tunnel once fully off screen.
    pub fn wrapped(self) -> Self {
        let x = if self.x > WINDOW_WIDTH + TILE {
            -TILE
        } else if self.x < -TILE {
            WINDOW_WIDTH + TILE
        } else {
            self.x
        };
        Self::new(x, self.y)
    }

    /// Whether two actor boxes anchored at these points overlap.
    pub fn overlaps(self, other: Self) -> bool {
        (other.x - self.x).abs() < TILE && (other.y - self.y).abs() < TILE
    }

    /// Where the 32px sprite of an actor at this point is drawn.
    pub fn sprite_origin(self) -> Self {
        let inset = (SPRITE - TILE) / 2;
        Self::new(self.x - inset, self.y - inset)
    }
}
