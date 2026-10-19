use crate::direction::Direction;
use crate::geometry::Point;
use crate::gfx::{Gfx, Sprite};
use crate::maze::{self, Maze};

/// Pixels moved per frame, one pixel at a time.
const SPEED: u32 = 2;
/// The mouth sprite changes every 12 pixels moved, cycling through 3 frames.
const MOUTH_STEPS_PER_FRAME: u32 = 12;
const MOUTH_FRAMES: u32 = 3;
/// Running into a wall shows the last mouth frame.
const MOUTH_BLOCKED: u32 = 32;
/// The death animation has 10 frames, each shown for 10 rendered frames.
const DEATH_TICKS_PER_FRAME: u32 = 10;
const DEATH_FRAMES: u32 = 10;

/// Player input: the direction being followed, plus the latest requested
/// turn, which is taken as soon as the maze allows it.
pub struct Steering {
    current: Direction,
    queued: Option<Direction>,
}

impl Steering {
    pub fn new() -> Self {
        Self {
            current: Direction::Right,
            queued: None,
        }
    }

    pub fn request(&mut self, direction: Direction) {
        self.queued = Some(direction);
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

pub struct Pacman {
    pub pos: Point,
    pub direction: Direction,
    pub alive: bool,
    pub energized: bool,
    death_animation_done: bool,
    mouth_counter: u32,
    death_counter: u32,
}

impl Pacman {
    pub fn new() -> Self {
        Self {
            pos: Point::default(),
            direction: Direction::Right,
            alive: true,
            energized: false,
            death_animation_done: false,
            mouth_counter: 0,
            death_counter: 0,
        }
    }

    pub fn respawn(&mut self) {
        self.pos = maze::spawn_point(b'0');
        self.energized = false;
        self.reset_mouth();
    }

    pub fn revive(&mut self) {
        self.alive = true;
        self.death_animation_done = false;
    }

    pub fn death_animation_done(&self) -> bool {
        self.death_animation_done
    }

    pub fn reset_mouth(&mut self) {
        self.mouth_counter = 0;
    }

    pub fn update(&mut self, steering: &mut Steering, maze: &Maze) {
        for _ in 0..SPEED {
            if maze.blocks(self.pos.step(steering.current), false) {
                self.mouth_counter = MOUTH_BLOCKED;
            } else {
                self.advance(steering.current);
            }
            if let Some(turn) = steering.queued.filter(|&turn| turn != steering.current)
                && !maze.blocks(self.pos.step(turn), false)
            {
                self.advance(turn);
                steering.current = turn;
                steering.queued = None;
            }
            self.pos = self.pos.wrapped();
        }
    }

    fn advance(&mut self, direction: Direction) {
        self.mouth_counter += 1;
        if self.mouth_counter / MOUTH_STEPS_PER_FRAME >= MOUTH_FRAMES {
            self.mouth_counter = 0;
        }
        self.pos = self.pos.step(direction);
        self.direction = direction;
    }

    /// Draws Pac-Man; while dead this also advances the death animation.
    pub fn draw(&mut self, gfx: &mut Gfx) -> Result<(), String> {
        let at = self.pos.sprite_origin();
        let angle = match self.direction {
            Direction::Right => 0.0,
            Direction::Down => 90.0,
            Direction::Left => 180.0,
            Direction::Up => 270.0,
        };
        if self.alive {
            let frame = self.mouth_counter / MOUTH_STEPS_PER_FRAME;
            return gfx.sprite(Sprite::PacMan, frame, at, angle);
        }
        let frame = self.death_counter / DEATH_TICKS_PER_FRAME;
        gfx.sprite(Sprite::PacManDeath, frame, at, angle)?;
        self.death_counter += 1;
        if self.death_counter / DEATH_TICKS_PER_FRAME >= DEATH_FRAMES {
            self.death_animation_done = true;
            self.death_counter = 0;
        }
        Ok(())
    }
}
