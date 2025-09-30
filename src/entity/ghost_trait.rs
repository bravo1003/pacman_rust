use crate::entity::Facing;
use crate::board::{BlockType, Direction, EntityType};
use crate::entity::{BaseEntity, Entity};
use crate::entity::pacman::Pacman;
use crate::position::Position;
use crate::texture::GameTexture;
use crate::{BLOCK_SIZE_24, BLOCK_SIZE_32, BLUE, RED, WHITE, WINDOW_WIDTH};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;

pub trait GhostBehavior<'a> {
    fn get_ghost_type(&self) -> GhostType;
    fn get_scatter_target(&self) -> Position;
    fn calculate_target(
        &mut self,
        pacman_pos: Position,
        pacman_dir: Direction,
        blinky_pos: Option<Position>,
    );
    fn get_can_use_door(&self) -> bool;
    fn set_can_use_door(&mut self, can_use_door: bool);
    fn get_ghost_mut(&mut self) -> &mut Ghost<'a>;
    fn get_ghost(&self) -> &Ghost<'a>;

    fn update_pos(
        &mut self,
        actual_map: &[BlockType],
        pacman: &Pacman,
        blinky_pos: Option<Position>,
        timed_status: bool,
    ) {
        let pacman_pos = pacman.get_position();
        let pacman_dir = pacman.get_direction();

        let speed = {
            let ghost = self.get_ghost_mut();
            ghost.update_speed(pacman.is_energized());
            ghost.update_status(pacman.is_energized(), timed_status);
            ghost.entity.get_speed()
        };

        for _ in 0..speed {
            let should_calculate = {
                let ghost = self.get_ghost_mut();
                ghost.should_calculate_normal_target(pacman.is_energized())
            };

            {
                let ghost = self.get_ghost_mut();
                ghost.update_facing(pacman.is_energized());
            }

            if should_calculate {
                self.calculate_target(pacman_pos, pacman_dir, blinky_pos);
            }

            {
                let ghost = self.get_ghost_mut();
                ghost.calculate_direction(actual_map);
                ghost.entity.move_entity(ghost.entity.get_direction());
                ghost.entity.check_wrap();
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GhostType {
    Blinky,
    Pinky,
    Inky,
    Clyde,
}

pub const GHOST_BODY_FRAMES: usize = 2;
pub const GHOST_EYE_FRAMES: usize = 5;

pub struct Ghost<'a> {
    pub entity: BaseEntity,
    pub body: GameTexture<'a>,
    pub eyes: GameTexture<'a>,
    pub ghost_body_sprite_clips: [Rect; GHOST_BODY_FRAMES],
    pub ghost_eye_sprite_clips: [Rect; GHOST_EYE_FRAMES],
    pub color: Color,
    pub current_body_frame: u8,
    pub can_use_door: bool,
    pub status: bool,
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

        body.load_from_file(texture_creator, "assets/GhostBody32.png")?;
        eyes.load_from_file(texture_creator, "assets/GhostEyes32.png")?;

        let ghost_body_sprite_clips = [
            Rect::new(0, 0, BLOCK_SIZE_32 as u32, BLOCK_SIZE_32 as u32),
            Rect::new(BLOCK_SIZE_32 as i32, 0, BLOCK_SIZE_32, BLOCK_SIZE_32),
        ];

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

    pub fn is_in_energized_home_containment(&self, pacman_energized: bool) -> bool {
        if !pacman_energized || !self.entity.is_alive() {
            return false;
        }

        let x = self.entity.position.x;
        let y = self.entity.position.y;

        if x > (11 * BLOCK_SIZE_24) as i16 && x < (17 * BLOCK_SIZE_24) as i16 {
            if y > (14 * BLOCK_SIZE_24) as i16 && y < (18 * BLOCK_SIZE_24) as i16 {
                return true;
            }
        }
        false
    }

    pub fn is_home(&self) -> bool {
        let x = self.entity.position.x;
        let y = self.entity.position.y;

        if x > (11 * BLOCK_SIZE_24) as i16 && x < (17 * BLOCK_SIZE_24) as i16 {
            if y > (15 * BLOCK_SIZE_24) as i16 && y < (18 * BLOCK_SIZE_24) as i16 {
                return true;
            }
        }
        false
    }

    pub fn should_calculate_normal_target(&mut self, pacman_energized: bool) -> bool {
        if !self.entity.is_alive() {
            self.can_use_door = true;
            self.target = self.home;

            if self.entity.position.x == self.home.x && self.entity.position.y == self.home.y {
                self.entity.mod_life_statement(true);
            } else {
                let dx = (self.entity.position.x - self.home.x).abs();
                let dy = (self.entity.position.y - self.home.y).abs();
                if dx <= 2 && dy <= 2 {
                    self.entity.mod_life_statement(true);
                    self.entity.set_position(self.home);
                }
            }
            return false;
        }

        if self.is_home() && pacman_energized {
            if self.entity.position.x == self.home.x && self.entity.position.y == self.home.y {
                self.target.y = self.home.y - BLOCK_SIZE_24 as i16;
            } else if self.entity.position.x == self.home.x
                && self.entity.position.y == self.home.y - BLOCK_SIZE_24 as i16
            {
                self.target.y = self.home.y;
            }
            return false;
        }

        if self.is_home() && self.entity.is_alive() {
            self.can_use_door = true;
            self.target = self.door_target;
            return false;
        }

        self.can_use_door = false;
        match self.status {
            false => true,
            true => {
                self.target = self.scatter_target;
                false
            }
        }
    }

    pub fn update_speed(&mut self, pacman_is_energized: bool) {
        if !self.entity.is_alive() && self.entity.get_speed() != 6 {
            self.entity.mod_speed(6);
            return;
        }

        if self.is_home() && self.entity.is_alive() {
            if self.entity.get_speed() != 2 {
                self.entity.mod_speed(2);
            }
            return;
        }

        if pacman_is_energized {
            if self.entity.get_speed() != 1 {
                self.entity.mod_speed(1);
            }
        } else {
            if self.entity.get_speed() != 2 {
                self.entity.mod_speed(2);
            }
        }
    }

    pub fn update_status(&mut self, pacman_is_energized: bool, timed_status: bool) {
        if pacman_is_energized {
            if !self.status {
                self.status = true;
            }
            return;
        }

        match timed_status {
            false => {
                if self.status {
                    self.status = false;
                }
            }
            true => {
                if !self.status {
                    self.status = true;
                }
            }
        }
    }

    pub fn update_facing(&mut self, pacman_is_energized: bool) {
        if self.is_home() {
            match self.entity.get_direction() {
                Direction::Down => self.entity.set_facing(Direction::Down),
                _ => self.entity.set_facing(Direction::Up),
            }
            return;
        }

        if pacman_is_energized {
            if !self.entity.is_alive() {
                self.entity.set_facing(self.entity.get_direction());
            } else {
                // Set to scared facing (special case for energized ghosts)
                self.entity.facing = Facing::Scared;
            }
            return;
        }

        self.entity.set_facing(self.entity.get_direction());
    }

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

        if pacman_is_energized
            && self.entity.is_alive()
            && !self.is_in_energized_home_containment(pacman_is_energized)
        {
            self.body.set_color(BLUE.r, BLUE.g, BLUE.b)?;

            if ghost_timer_ticks > (ghost_timer_target as u128 - 2000) {
                if (ghost_timer_ticks / 250) % 2 == 1 {
                    self.body.set_color(WHITE.r, WHITE.g, WHITE.b)?;
                    self.eyes.set_color(RED.r, RED.g, RED.b)?;
                } else {
                    self.eyes.set_color(WHITE.r, WHITE.g, WHITE.b)?;
                }
            } else {
                self.eyes.set_color(WHITE.r, WHITE.g, WHITE.b)?;
            }
        } else {
            self.body
                .set_color(self.color.r, self.color.g, self.color.b)?;
            self.eyes.set_color(WHITE.r, WHITE.g, WHITE.b)?;
        }

        if self.entity.is_alive() {
            let body_clip = &self.ghost_body_sprite_clips
                [(self.current_body_frame / 8) as usize % GHOST_BODY_FRAMES];
            self.body.render(canvas, x, y, Some(*body_clip))?;
        }

        let eye_frame = self.entity.get_facing().as_u8() as usize;
        let eye_frame = if eye_frame >= GHOST_EYE_FRAMES {
            0
        } else {
            eye_frame
        };
        let eye_clip = &self.ghost_eye_sprite_clips[eye_frame];
        self.eyes.render(canvas, x, y, Some(*eye_clip))?;

        self.current_body_frame = (self.current_body_frame + 1) % (GHOST_BODY_FRAMES as u8 * 8);
        Ok(())
    }
}
