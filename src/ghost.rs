use sdl2::pixels::Color;

use crate::direction::Direction;
use crate::geometry::{Point, TILE, WINDOW_WIDTH};
use crate::gfx::{self, Gfx, Sprite};
use crate::maze::{self, Maze};
use crate::pacman::Pacman;

const NORMAL_SPEED: u32 = 2;
const FRIGHTENED_SPEED: u32 = 1;
const DEAD_SPEED: u32 = 6;
/// Leaving the house means heading for the tile just above the door.
const DOOR_TARGET: Point = Point::new(13 * TILE + TILE / 2, 15 * TILE);
/// Frightened ghosts flash during the last two seconds of the fright.
const FLASH_WARNING_MS: u32 = 2000;
const FLASH_PERIOD_MS: u32 = 250;
/// The body animation has 6 frames, each shown for 6 rendered frames.
const BODY_TICKS_PER_FRAME: u32 = 6;
const BODY_FRAMES: u32 = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GhostKind {
    Blinky,
    Pinky,
    Inky,
    Clyde,
}

impl GhostKind {
    fn color(self) -> Color {
        match self {
            Self::Blinky => Color::RGB(255, 0, 0),
            Self::Pinky => Color::RGB(255, 192, 203),
            Self::Inky => Color::RGB(0, 192, 255),
            Self::Clyde => Color::RGB(255, 128, 0),
        }
    }

    fn spawn_marker(self) -> u8 {
        match self {
            Self::Blinky => b'1',
            Self::Inky => b'2',
            Self::Pinky => b'3',
            Self::Clyde => b'4',
        }
    }

    /// Where a dead ghost goes to be revived.
    fn home(self) -> Point {
        let col = match self {
            Self::Blinky | Self::Pinky => 13,
            Self::Inky => 11,
            Self::Clyde => 15,
        };
        Point::new(col * TILE + TILE / 2, 17 * TILE + TILE / 2)
    }

    /// The corner each ghost retreats to while scattering.
    fn scatter_target(self) -> Point {
        let (col, row) = match self {
            Self::Blinky => (25, 0),
            Self::Pinky => (2, 0),
            Self::Inky => (26, 35),
            Self::Clyde => (0, 35),
        };
        Point::new(col * TILE + TILE / 2, row * TILE + TILE / 2)
    }

    fn starting_eyes(self) -> Eyes {
        match self {
            Self::Blinky => Eyes::Look(Direction::Right),
            _ => Eyes::Look(Direction::Up),
        }
    }
}

#[derive(Clone, Copy)]
enum Eyes {
    Look(Direction),
    Frightened,
}

impl Eyes {
    fn frame(self) -> u32 {
        match self {
            Self::Look(Direction::Right) => 0,
            Self::Look(Direction::Up) => 1,
            Self::Look(Direction::Left) => 2,
            Self::Look(Direction::Down) => 3,
            Self::Frightened => 4,
        }
    }
}

pub struct Ghost {
    kind: GhostKind,
    pub pos: Point,
    pub alive: bool,
    direction: Direction,
    speed: u32,
    scatter: bool,
    can_use_door: bool,
    target: Point,
    eyes: Eyes,
    body_counter: u32,
}

impl Ghost {
    pub fn new(kind: GhostKind) -> Self {
        Self {
            kind,
            pos: Point::default(),
            alive: true,
            direction: Direction::Right,
            speed: NORMAL_SPEED,
            scatter: false,
            can_use_door: false,
            target: Point::default(),
            eyes: Eyes::Look(Direction::Right),
            body_counter: 0,
        }
    }

    pub fn respawn(&mut self) {
        self.pos = maze::spawn_point(self.kind.spawn_marker());
        self.alive = true;
        self.eyes = self.kind.starting_eyes();
    }

    /// Moves one frame. `blinky` is Blinky's position, which Inky aims by.
    pub fn update(&mut self, maze: &Maze, pacman: &Pacman, blinky: Point, scatter_phase: bool) {
        self.update_speed(pacman.energized);
        self.scatter = pacman.energized || scatter_phase;
        for _ in 0..self.speed {
            self.update_eyes(pacman.energized);
            self.retarget(pacman, blinky);
            self.choose_direction(maze);
            self.pos = self.pos.step(self.direction).wrapped();
        }
    }

    fn update_speed(&mut self, energized: bool) {
        // As in the original, a dead ghost alternates between DEAD_SPEED and
        // the regular speed on successive frames.
        if !self.alive && self.speed != DEAD_SPEED {
            self.speed = DEAD_SPEED;
        } else if energized {
            self.speed = FRIGHTENED_SPEED;
        } else {
            self.speed = NORMAL_SPEED;
        }
    }

    fn update_eyes(&mut self, energized: bool) {
        self.eyes = if self.is_home() {
            let down = self.direction == Direction::Down;
            Eyes::Look(if down { Direction::Down } else { Direction::Up })
        } else if energized && self.alive {
            Eyes::Frightened
        } else {
            Eyes::Look(self.direction)
        };
    }

    fn is_home(&self) -> bool {
        (11 * TILE + 1..17 * TILE).contains(&self.pos.x)
            && (15 * TILE + 1..18 * TILE).contains(&self.pos.y)
    }

    fn retarget(&mut self, pacman: &Pacman, blinky: Point) {
        let home = self.kind.home();
        if !self.alive {
            self.can_use_door = true;
            self.target = home;
            if self.pos == home {
                self.alive = true;
            }
        } else if self.is_home() && pacman.energized {
            // Bob up and down inside the house until the fright is over.
            let above_home = Point::new(home.x, home.y - TILE);
            if self.pos == home {
                self.target.y = above_home.y;
            } else if self.pos == above_home {
                self.target.y = home.y;
            }
        } else if self.is_home() {
            self.can_use_door = true;
            self.target = DOOR_TARGET;
        } else {
            self.can_use_door = false;
            self.target = if self.scatter {
                self.kind.scatter_target()
            } else {
                self.chase_target(pacman, blinky)
            };
        }
    }

    fn chase_target(&self, pacman: &Pacman, blinky: Point) -> Point {
        match self.kind {
            GhostKind::Blinky => pacman.pos,
            GhostKind::Pinky => pacman.pos.offset(pacman.direction, 4 * TILE),
            GhostKind::Inky => {
                let pivot = pacman.pos.offset(pacman.direction, 2 * TILE);
                Point::new(2 * pivot.x - blinky.x, 2 * pivot.y - blinky.y)
            }
            GhostKind::Clyde => {
                if distance(self.pos, pacman.pos) > (8 * TILE) as f32 {
                    pacman.pos
                } else {
                    self.kind.scatter_target()
                }
            }
        }
    }

    /// Takes the open direction closest to the target without reversing,
    /// unless reversing is the only way out.
    fn choose_direction(&mut self, maze: &Maze) {
        let mut options: Vec<(f32, Direction)> = Direction::ALL
            .into_iter()
            .map(|direction| (self.pos.step(direction), direction))
            .filter(|&(next, _)| !maze.blocks(next, self.can_use_door))
            .map(|(next, direction)| (distance(next, self.target), direction))
            .collect();
        if let &[(_, only)] = options.as_slice() {
            self.direction = only;
            return;
        }
        // The original's exchange sort, kept as is: the order it leaves equal
        // distances in decides which way a ghost turns.
        for i in 0..options.len() {
            for j in 0..options.len() {
                if options[i].0 < options[j].0 {
                    options.swap(i, j);
                }
            }
        }
        let reverse = self.direction.opposite();
        if let Some(&(_, direction)) = options.iter().find(|&&(_, d)| d != reverse) {
            self.direction = direction;
        }
    }

    /// Draws the ghost and advances its body animation.
    pub fn draw(
        &mut self,
        gfx: &mut Gfx,
        energized: bool,
        phase_ms: u32,
        scatter_time: u32,
    ) -> Result<(), String> {
        let (body, eyes) = if energized && self.alive && !self.is_home() {
            let flashing =
                phase_ms > scatter_time - FLASH_WARNING_MS && (phase_ms / FLASH_PERIOD_MS) % 2 == 1;
            if flashing {
                (gfx::WHITE, gfx::RED)
            } else {
                (gfx::BLUE, gfx::WHITE)
            }
        } else {
            (self.kind.color(), gfx::WHITE)
        };
        let at = self.pos.sprite_origin();
        if self.alive {
            let frame = self.body_counter / BODY_TICKS_PER_FRAME;
            gfx.tinted_sprite(Sprite::GhostBody, frame, at, body)?;
        }
        gfx.tinted_sprite(Sprite::GhostEyes, self.eyes.frame(), at, eyes)?;
        self.body_counter += 1;
        if self.body_counter / BODY_TICKS_PER_FRAME >= BODY_FRAMES {
            self.body_counter = 0;
        }
        Ok(())
    }
}

/// Euclidean distance where the horizontal leg may go through the tunnel,
/// rounded to `f32` exactly like the original so that ties match.
fn distance(a: Point, b: Point) -> f32 {
    let mut dx = (a.x - b.x).abs();
    if dx > WINDOW_WIDTH / 2 {
        dx = WINDOW_WIDTH - dx;
    }
    let dy = a.y - b.y;
    f64::from(dx * dx + dy * dy).sqrt() as f32
}
