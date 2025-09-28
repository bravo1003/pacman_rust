use crate::board::{BlockType, Direction, EntityType};
use crate::entity::{BaseEntity, Entity};
use crate::position::Position;
use crate::texture::LTexture;
use crate::{BLOCK_SIZE_24, BLOCK_SIZE_32, BOARD_WIDTH};
use sdl2::rect::Rect;
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;

// Constants from C++ version
const LIVING_PAC_FRAMES: usize = 3;
const DEATH_PAC_FRAMES: usize = 10;

pub struct Pacman<'a> {
    pub entity: BaseEntity,

    // Textures for sprites
    living_pac: LTexture<'a>,
    death_pac: LTexture<'a>,

    // Animation frames (like C++ SpriteClips)
    living_pac_sprite_clips: [Rect; LIVING_PAC_FRAMES],
    death_pac_sprite_clips: [Rect; DEATH_PAC_FRAMES],

    // Animation state
    curr_living_pac_frame: u8,
    curr_death_pac_frame: u8,

    // Game state
    energy_status: bool,
    dead_animation_statement: bool,
}

impl<'a> Pacman<'a> {
    pub fn new(
        texture_creator: &'a TextureCreator<WindowContext>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut pacman = Pacman {
            entity: BaseEntity::new(EntityType::PacMan),
            living_pac: LTexture::new(),
            death_pac: LTexture::new(),
            living_pac_sprite_clips: [Rect::new(0, 0, 0, 0); LIVING_PAC_FRAMES],
            death_pac_sprite_clips: [Rect::new(0, 0, 0, 0); DEATH_PAC_FRAMES],
            curr_living_pac_frame: 0,
            curr_death_pac_frame: 0,
            energy_status: false,
            dead_animation_statement: false,
        };

        // Load textures (like C++ constructor)
        pacman
            .living_pac
            .load_from_file(texture_creator, "assets/PacMan32.png")?;
        pacman
            .death_pac
            .load_from_file(texture_creator, "assets/GameOver32.png")?;

        // Initialize sprite frames (like C++ InitFrames)
        pacman.init_frames();

        Ok(pacman)
    }

    // Initialize sprite frames (matching C++ InitFrames function)
    fn init_frames(&mut self) {
        // Living Pac frames
        let mut counter = 0;
        for i in 0..LIVING_PAC_FRAMES {
            self.living_pac_sprite_clips[i] = Rect::new(counter, 0, BLOCK_SIZE_32, BLOCK_SIZE_32);
            counter += BLOCK_SIZE_32 as i32;
        }

        // Death Pac frames
        counter = 0;
        for i in 0..DEATH_PAC_FRAMES {
            self.death_pac_sprite_clips[i] = Rect::new(counter, 0, BLOCK_SIZE_32, BLOCK_SIZE_32);
            counter += BLOCK_SIZE_32 as i32;
        }
    }

    // Update position based on input (like C++ UpdatePos)
    pub fn update_pos(&mut self, mover: &mut Vec<Direction>, actual_map: &[BlockType]) {
        if mover.is_empty() {
            return;
        }

        for _ in 0..self.entity.get_speed() {
            let (temp_x, temp_y) = self.entity.get_possible_position(mover[0]);

            if !self
                .entity
                .wall_collision(temp_x, temp_y, actual_map, false)
            {
                self.update_current_living_pac_frame();
                self.entity.move_entity(mover[0]);
                self.set_facing(mover[0]);
                self.entity.mod_direction(mover[0]);
            } else {
                self.wall_collision_frame();
            }

            // Handle second input if different from first (like C++ version)
            if mover.len() > 1 && mover[0] != mover[1] {
                let (temp_x, temp_y) = self.entity.get_possible_position(mover[1]);

                if !self
                    .entity
                    .wall_collision(temp_x, temp_y, actual_map, false)
                {
                    self.update_current_living_pac_frame();
                    self.entity.move_entity(mover[1]);
                    self.set_facing(mover[1]);
                    self.entity.mod_direction(mover[1]);
                    mover.remove(0); // Remove first element (like C++ mover.erase)
                }
            }

            self.entity.check_wrap();
        }
    }

    // Check food collision and consumption (like C++ FoodCollision)
    pub fn food_collision(&self, actual_map: &mut [BlockType]) -> u8 {
        let cell_x = self.entity.get_x() as f32 / BLOCK_SIZE_24 as f32;
        let cell_y = self.entity.get_y() as f32 / BLOCK_SIZE_24 as f32;

        for side_dir in 0..4 {
            let board_pos = self.entity.char_board_pos(side_dir, cell_x, cell_y);
            let board_x = board_pos.get_x() as usize;
            let board_y = board_pos.get_y() as usize;

            if board_y < crate::BOARD_HEIGHT && board_x < BOARD_WIDTH {
                let index = BOARD_WIDTH * board_y + board_x;

                if index < actual_map.len() {
                    match actual_map[index] {
                        BlockType::Pellet => {
                            actual_map[index] = BlockType::Nothing;
                            return 0; // Pellet eaten
                        }
                        BlockType::Energizer => {
                            actual_map[index] = BlockType::Nothing;
                            return 1; // Energizer eaten
                        }
                        _ => {}
                    }
                }
            }
        }
        2 // No food eaten
    }

    // Energy status methods
    pub fn is_energized(&self) -> bool {
        self.energy_status
    }

    pub fn change_energy_status(&mut self, new_energy_status: bool) {
        self.energy_status = new_energy_status;
    }

    // Set facing direction based on movement (like C++ SetFacing)
    fn set_facing(&mut self, mover: Direction) {
        match mover {
            Direction::Right => self.entity.mod_facing(0),
            Direction::Up => self.entity.mod_facing(3),
            Direction::Left => self.entity.mod_facing(2),
            Direction::Down => self.entity.mod_facing(1),
            Direction::Nowhere => {}
        }
    }

    // Animation methods (like C++ version)
    pub fn is_dead_animation_ended(&self) -> bool {
        self.dead_animation_statement
    }

    pub fn mod_dead_animation_statement(&mut self, new_dead_animation_statement: bool) {
        self.dead_animation_statement = new_dead_animation_statement;
    }

    fn update_current_living_pac_frame(&mut self) {
        self.curr_living_pac_frame += 1;
        if self.curr_living_pac_frame / ((LIVING_PAC_FRAMES * 4) as u8) >= LIVING_PAC_FRAMES as u8 {
            self.curr_living_pac_frame = 0;
        }
    }

    pub fn reset_current_living_frame(&mut self) {
        self.curr_living_pac_frame = 0;
    }

    fn wall_collision_frame(&mut self) {
        self.curr_living_pac_frame = 32;
    }

    // Entity trait delegation
    pub fn is_alive(&self) -> bool {
        self.entity.is_alive()
    }

    pub fn mod_life_statement(&mut self, new_life_statement: bool) {
        self.entity.mod_life_statement(new_life_statement);
    }

    pub fn get_position(&self) -> Position {
        self.entity.get_position()
    }

    pub fn set_position(&mut self, position: Position) {
        self.entity.set_position(position);
    }

    pub fn get_x(&self) -> i16 {
        self.entity.get_x()
    }

    pub fn get_y(&self) -> i16 {
        self.entity.get_y()
    }

    pub fn is_colliding(&self, other: Position) -> bool {
        self.entity.is_colliding(other)
    }

    // Draw Pacman (like C++ Draw method)
    pub fn draw(&mut self, canvas: &mut WindowCanvas) -> Result<(), Box<dyn std::error::Error>> {
        if self.entity.is_alive() {
            // Draw living Pacman with facing direction (like C++ version)
            let current_clip = &self.living_pac_sprite_clips
                [(self.curr_living_pac_frame / ((LIVING_PAC_FRAMES * 4) as u8)) as usize];

            self.living_pac.render_with_facing(
                canvas,
                (self.entity.get_x() - 4) as i32, // Offset like C++ version
                (self.entity.get_y() - 4) as i32,
                self.entity.get_facing(), // Use facing direction
                Some(*current_clip),
            )?;
        } else {
            // Draw death animation (facing doesn't matter for death animation)
            let current_clip = &self.death_pac_sprite_clips
                [(self.curr_death_pac_frame / DEATH_PAC_FRAMES as u8) as usize];

            self.death_pac.render_with_facing(
                canvas,
                (self.entity.get_x() - 4) as i32,
                (self.entity.get_y() - 4) as i32,
                self.entity.get_facing(),
                Some(*current_clip),
            )?;

            self.curr_death_pac_frame += 1;
            if self.curr_death_pac_frame / DEATH_PAC_FRAMES as u8 >= DEATH_PAC_FRAMES as u8 {
                self.dead_animation_statement = true;
                self.curr_death_pac_frame = 0;
            }
        }

        Ok(())
    }
}
