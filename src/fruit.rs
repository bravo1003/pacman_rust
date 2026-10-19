use crate::geometry::{Point, TILE};
use crate::gfx::{Gfx, Sprite};
use crate::timer::Timer;

pub const POSITION: Point = Point::new(13 * TILE + TILE / 2, 20 * TILE);
/// Points for each fruit; the fruit index doubles as its sprite frame.
const POINTS: [u32; 8] = [100, 300, 500, 700, 1000, 2000, 3000, 5000];
/// A fruit appears after this much food has been eaten in a level.
const APPEARS_AFTER: [u32; 2] = [70, 200];
const STAYS_MS: u32 = 9000;
const POINTS_SHOWN_MS: u32 = 1000;

pub struct Fruit {
    kind: usize,
    food_eaten: u32,
    shown: Timer,
    points_shown: Timer,
}

impl Fruit {
    pub fn new() -> Self {
        Self {
            kind: 0,
            food_eaten: 0,
            shown: Timer::default(),
            points_shown: Timer::default(),
        }
    }

    /// Picks the fruit for a level: a new one every three levels.
    pub fn set_level(&mut self, level: u32) {
        self.kind = ((level as usize - 1) / 3).min(POINTS.len() - 1);
    }

    pub fn points(&self) -> u32 {
        POINTS[self.kind]
    }

    pub fn food_eaten(&mut self, now: u32) {
        self.food_eaten += 1;
        if APPEARS_AFTER.contains(&self.food_eaten) && !self.shown.is_started() {
            self.shown.start(now);
        }
    }

    pub fn reset_food_count(&mut self) {
        self.food_eaten = 0;
    }

    pub fn is_shown(&self) -> bool {
        self.shown.is_started()
    }

    pub fn has_expired(&self, now: u32) -> bool {
        self.shown.ticks(now) > STAYS_MS
    }

    pub fn despawn(&mut self) {
        self.shown.reset();
    }

    pub fn eat(&mut self, now: u32) {
        self.points_shown.start(now);
        self.despawn();
    }

    pub fn draw(&self, gfx: &mut Gfx, now: u32) -> Result<(), String> {
        if self.shown.is_started() {
            gfx.sprite(
                Sprite::Fruit,
                self.kind as u32,
                POSITION.sprite_origin(),
                0.0,
            )?;
        }
        if self.points_shown.is_started() && self.points_shown.ticks(now) < POINTS_SHOWN_MS {
            gfx.small_number(self.points(), Point::new(POSITION.x, POSITION.y - TILE / 2))?;
        }
        Ok(())
    }
}
