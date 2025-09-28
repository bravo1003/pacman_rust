use crate::board::{BlockType, Direction, EntityType};
use crate::entity::{BaseEntity, Entity};
use crate::position::Position;
use crate::texture::GameTexture;
use crate::{BLOCK_SIZE_24, BLOCK_SIZE_32, WINDOW_WIDTH};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;

// Common ghost behavior trait
pub trait GhostBehavior<'a> {
    fn get_ghost_type(&self) -> GhostType;
    fn get_scatter_target(&self) -> Position;
    fn calculate_target(&mut self, pacman_pos: Position, blinky_pos: Option<Position>);
    fn get_can_use_door(&self) -> bool;
    fn set_can_use_door(&mut self, can_use_door: bool);
    fn get_ghost_mut(&mut self) -> &mut Ghost<'a>;
    fn get_ghost(&self) -> &Ghost<'a>;
}

// Ghost types enum
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GhostType {
    Blinky,
    Pinky,
    Inky,
    Clyde,
}

// Common ghost constants
pub const GHOST_BODY_FRAMES: usize = 2;
pub const GHOST_EYE_FRAMES: usize = 5; // 4 directional + 1 frightened

// Base Ghost struct that all ghosts will use
pub struct Ghost<'a> {
    pub entity: BaseEntity,
    pub body: GameTexture<'a>,
    pub eyes: GameTexture<'a>,
    pub ghost_body_sprite_clips: [Rect; GHOST_BODY_FRAMES],
    pub ghost_eye_sprite_clips: [Rect; GHOST_EYE_FRAMES],
    pub color: Color,
    pub current_body_frame: u8,
    pub can_use_door: bool,
    pub status: bool, // false = chase, true = scatter
    pub target: Position,
    pub scatter_target: Position,
    pub door_target: Position,
    pub home: Position,
}

impl<'a> Ghost<'a> {
    pub fn new(
        color: Color,
        identity: EntityType,
        scatter_target: Position,
        home_position: Position,
        texture_creator: &'a sdl2::render::TextureCreator<sdl2::video::WindowContext>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut body = GameTexture::new();
        let mut eyes = GameTexture::new();

        // Load ghost textures
        body.load_from_file(texture_creator, "assets/GhostBody32.png")?;
        eyes.load_from_file(texture_creator, "assets/GhostEyes32.png")?;

        // Initialize body sprite clips
        let ghost_body_sprite_clips = [
            Rect::new(0, 0, BLOCK_SIZE_32 as u32, BLOCK_SIZE_32 as u32),
            Rect::new(BLOCK_SIZE_32 as i32, 0, BLOCK_SIZE_32, BLOCK_SIZE_32),
        ];

        // Initialize eye sprite clips (Right, Up, Left, Down, Frightened)
        let ghost_eye_sprite_clips = [
            Rect::new(0, 0, BLOCK_SIZE_32 as u32, BLOCK_SIZE_32 as u32),
            Rect::new(
                BLOCK_SIZE_32 as i32,
                0,
                BLOCK_SIZE_32 as u32,
                BLOCK_SIZE_32 as u32,
            ),
            Rect::new(
                2 * BLOCK_SIZE_32 as i32,
                0,
                BLOCK_SIZE_32 as u32,
                BLOCK_SIZE_32 as u32,
            ),
            Rect::new(
                3 * BLOCK_SIZE_32 as i32,
                0,
                BLOCK_SIZE_32 as u32,
                BLOCK_SIZE_32 as u32,
            ),
            Rect::new(
                4 * BLOCK_SIZE_32 as i32,
                0,
                BLOCK_SIZE_32 as u32,
                BLOCK_SIZE_32 as u32,
            ),
        ];

        // Create entity starting at home position
        let mut entity = BaseEntity::new(identity);
        entity.position = home_position;

        Ok(Ghost {
            entity,
            body,
            eyes,
            ghost_body_sprite_clips,
            ghost_eye_sprite_clips,
            color,
            current_body_frame: 0,
            can_use_door: false,
            status: false,
            target: Position::new(0, 0),
            scatter_target,
            door_target: Position::new(
                (13 * BLOCK_SIZE_24 + BLOCK_SIZE_24 / 2) as i16,
                (15 * BLOCK_SIZE_24) as i16,
            ),
            home: home_position,
        })
    }

    // Common ghost methods

    /// Check if ghost is in home area (like C++ IsHome())
    pub fn is_home(&self) -> bool {
        let x = self.entity.position.x;
        let y = self.entity.position.y;

        // C++ logic: x > 11*BlockSize24 && x < 17*BlockSize24 && y > 15*BlockSize24 && y < 18*BlockSize24
        if x > (11 * BLOCK_SIZE_24) as i16 && x < (17 * BLOCK_SIZE_24) as i16 {
            if y > (15 * BLOCK_SIZE_24) as i16 && y < (18 * BLOCK_SIZE_24) as i16 {
                return true;
            }
        }
        false
    }

    /// Determines if normal AI target calculation should be used (like C++ IsTargetToCalculate())
    pub fn should_calculate_normal_target(&mut self, pacman_energized: bool) -> bool {
        // C++ Logic: Dead ghosts always target home and resurrect immediately when reaching it
        if !self.entity.is_alive() {
            self.can_use_door = true;
            self.target = self.home;
            if self.entity.position.x == self.home.x && self.entity.position.y == self.home.y {
                self.entity.mod_life_statement(true); // ✅ Always resurrect immediately (C++ behavior)
            }
            return false;
        }

        // C++ Logic: If alive, in home, and pacman is energized - do oscillation and stay trapped!
        if self.is_home() && pacman_energized {
            // Oscillate up and down within home (like C++ logic)
            if self.entity.position.x == self.home.x && self.entity.position.y == self.home.y {
                self.target.y = self.home.y - BLOCK_SIZE_24 as i16;
            } else if self.entity.position.x == self.home.x
                && self.entity.position.y == self.home.y - BLOCK_SIZE_24 as i16
            {
                self.target.y = self.home.y;
            }
            return false; // IMPORTANT: Prevent exit while energized!
        }

        // C++ Logic: If alive, in home, and NOT energized - allow exit via door
        if self.is_home() && self.entity.is_alive() {
            self.can_use_door = true;
            self.target = self.door_target;
            return false; // Don't use normal AI targeting
        }

        // Outside home - use normal AI (chase/scatter)
        self.can_use_door = false;
        match self.status {
            false => true, // Chase mode - calculate target
            true => {
                self.target = self.scatter_target;
                false // Scatter mode - use scatter target
            }
        }
    }

    pub fn is_target_to_calculate(&self, pacman_is_energized: bool) -> bool {
        if !self.entity.is_alive() {
            false
        } else if pacman_is_energized {
            self.entity.get_x() % BLOCK_SIZE_24 as i16 == 0
                && self.entity.get_y() % BLOCK_SIZE_24 as i16 == 0
        } else {
            match self.entity.get_x() % BLOCK_SIZE_24 as i16 {
                0 => self.entity.get_y() % BLOCK_SIZE_24 as i16 == 0,
                12 => self.entity.get_y() % BLOCK_SIZE_24 as i16 == 0,
                _ => false,
            }
        }
    }

    pub fn update_speed(&mut self, pacman_is_energized: bool) {
        // C++ UpdateSpeed logic
        if !self.entity.is_alive() && self.entity.get_speed() != 6 {
            self.entity.mod_speed(6); // Dead ghosts move fast to get home
            return;
        }

        if pacman_is_energized {
            if self.entity.get_speed() != 1 {
                self.entity.mod_speed(1); // Frightened ghosts move slower
            }
        } else {
            if self.entity.get_speed() != 2 {
                self.entity.mod_speed(2); // Normal ghost speed
            }
        }
    }

    pub fn update_status(&mut self, pacman_is_energized: bool, timed_status: bool) {
        // C++ UpdateStatus logic: if pacman is energized, force scatter mode (status = true)
        if pacman_is_energized {
            if !self.status {
                self.status = true; // Force scatter mode when pacman is energized
            }
            return;
        }

        // When not energized, use normal timed status (chase/scatter cycle)
        match timed_status {
            false => {
                if self.status {
                    self.status = false; // Switch to chase mode
                }
            }
            true => {
                if !self.status {
                    self.status = true; // Switch to scatter mode
                }
            }
        }
    }

    pub fn update_facing(&mut self, pacman_is_energized: bool) {
        if self.is_home() {
            match self.entity.get_direction() {
                Direction::Down => self.entity.mod_facing(3),
                _ => self.entity.mod_facing(1),
            }
            return;
        }

        if pacman_is_energized {
            if !self.entity.is_alive() {
                let facing = match self.entity.get_direction() {
                    Direction::Right => 0,
                    Direction::Up => 1,
                    Direction::Left => 2,
                    Direction::Down => 3,
                    Direction::Nowhere => 0,
                };
                self.entity.mod_facing(facing);
            } else {
                self.entity.mod_facing(4); // C++ uses facing 4 for frightened ghosts
            }
            return;
        }

        let facing = match self.entity.get_direction() {
            Direction::Right => 0,
            Direction::Up => 1,
            Direction::Left => 2,
            Direction::Down => 3,
            Direction::Nowhere => 0,
        };
        self.entity.mod_facing(facing);
    }

    // Direction calculation with bubble sort (common for all ghosts)
    pub fn calculate_direction(&mut self, actual_map: &[BlockType]) {
        let mut distances = Vec::new();
        let mut possible_directions = Vec::new();

        for i in 0..4 {
            let direction = match i {
                0 => Direction::Right,
                1 => Direction::Up,
                2 => Direction::Left,
                3 => Direction::Down,
                _ => Direction::Right,
            };

            let (x, y) = self.entity.get_possible_position(direction);

            if !self
                .entity
                .wall_collision(x, y, actual_map, self.can_use_door)
            {
                let mut dist_x = (x - self.target.get_x()).abs() as f32;
                if dist_x > (WINDOW_WIDTH / 2) as f32 {
                    dist_x = WINDOW_WIDTH as f32 - dist_x;
                }
                let dist = (dist_x.powi(2) + ((y - self.target.get_y()) as f32).powi(2)).sqrt();
                distances.push(dist);
                possible_directions.push(i);
            }
        }

        if possible_directions.len() == 1 {
            let direction = match possible_directions[0] {
                0 => Direction::Right,
                1 => Direction::Up,
                2 => Direction::Left,
                3 => Direction::Down,
                _ => Direction::Right,
            };
            self.entity.mod_direction(direction);
            return;
        }

        // Bubble sort
        for i in 0..distances.len() {
            for j in 0..distances.len() {
                if distances[i] < distances[j] {
                    distances.swap(i, j);
                    possible_directions.swap(i, j);
                }
            }
        }

        let current_numeric_dir = match self.entity.get_direction() {
            Direction::Right => 0,
            Direction::Up => 1,
            Direction::Left => 2,
            Direction::Down => 3,
            Direction::Nowhere => 0,
        };

        for &numeric_dir in &possible_directions {
            if numeric_dir != (current_numeric_dir + 2) % 4 {
                let direction = match numeric_dir {
                    0 => Direction::Right,
                    1 => Direction::Up,
                    2 => Direction::Left,
                    3 => Direction::Down,
                    _ => Direction::Right,
                };
                self.entity.mod_direction(direction);
                return;
            }
        }

        if !possible_directions.is_empty() {
            let direction = match possible_directions[0] {
                0 => Direction::Right,
                1 => Direction::Up,
                2 => Direction::Left,
                3 => Direction::Down,
                _ => Direction::Right,
            };
            self.entity.mod_direction(direction);
        }
    }

    pub fn draw(
        &mut self,
        canvas: &mut WindowCanvas,
        pacman_is_energized: bool,
        ghost_timer_ticks: u128,
        ghost_timer_target: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let x = (self.entity.get_x() - 4) as i32;
        let y = (self.entity.get_y() - 4) as i32;

        if pacman_is_energized && self.entity.is_alive() && !self.is_home() {
            // C++ Draw logic: set blue color and handle blinking near end
            self.body.set_color(0, 0, 255)?;

            // Blinking effect in last 2 seconds (like C++ mGhostTimer.GetTicks() > mTimerTarget - 2000)
            if ghost_timer_ticks > (ghost_timer_target as u128 - 2000) {
                // Blink every 250ms (like C++ (mGhostTimer.GetTicks() / 250) % 2 == 1)
                if (ghost_timer_ticks / 250) % 2 == 1 {
                    self.body.set_color(255, 255, 255)?; // White body
                    self.eyes.set_color(255, 0, 0)?; // Red eyes
                } else {
                    self.eyes.set_color(255, 255, 255)?; // White eyes (body stays blue)
                }
            } else {
                self.eyes.set_color(255, 255, 255)?; // White eyes
            }
        } else {
            self.body
                .set_color(self.color.r, self.color.g, self.color.b)?;
            self.eyes.set_color(255, 255, 255)?;
        }

        // Render body only if alive (like C++ version)
        if self.entity.is_alive() {
            let body_clip = &self.ghost_body_sprite_clips
                [(self.current_body_frame / 8) as usize % GHOST_BODY_FRAMES];
            self.body.render(canvas, x, y, Some(*body_clip))?;
        }

        // Always render eyes (like C++ version)
        let eye_frame = self.entity.get_facing() as usize;
        let eye_frame = if eye_frame >= GHOST_EYE_FRAMES {
            0
        } else {
            eye_frame
        };
        let eye_clip = &self.ghost_eye_sprite_clips[eye_frame];
        self.eyes.render(canvas, x, y, Some(*eye_clip))?;

        // Update animation frame
        self.current_body_frame = (self.current_body_frame + 1) % (GHOST_BODY_FRAMES as u8 * 8);
        Ok(())
    }
}
