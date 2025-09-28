use crate::board::{Board, BlockType, Direction};
use crate::pacman::Pacman;
use crate::{BOARD_HEIGHT, BOARD_WIDTH};
use sdl2::keyboard::Keycode;
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::ttf::{Font, Sdl2TtfContext};
use sdl2::video::WindowContext;

pub struct Game<'a> {
    board: Board<'a>,
    pacman: Pacman<'a>,
    actual_map: [BlockType; BOARD_HEIGHT * BOARD_WIDTH],
    is_game_started: bool,
    mover: Vec<Direction>,
}

impl<'a> Game<'a> {
    pub fn new(
        texture_creator: &'a TextureCreator<WindowContext>,
        ttf_context: &'a Sdl2TtfContext,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let board = Board::new(texture_creator, ttf_context)?;
        let mut pacman = Pacman::new(texture_creator)?;
        let mut actual_map = [BlockType::Nothing; BOARD_HEIGHT * BOARD_WIDTH];
        board.copy_board(&mut actual_map);
        
        // Position Pacman at starting position (like C++ ResetPosition)
        let start_pos = board.reset_position(crate::board::EntityType::PacMan);
        pacman.set_position(start_pos);

        Ok(Game {
            board,
            pacman,
            actual_map,
            is_game_started: true, // Start immediately for easier debugging
            mover: vec![Direction::Right], // Default direction
        })
    }

    pub fn handle_input(&mut self, keycode: Keycode) {
        match keycode {
            Keycode::Right | Keycode::D => {
                self.mover.push(Direction::Right);
            }
            Keycode::Up | Keycode::W => {
                self.mover.push(Direction::Up);
            }
            Keycode::Left | Keycode::A => {
                self.mover.push(Direction::Left);
            }
            Keycode::Down | Keycode::S => {
                self.mover.push(Direction::Down);
            }
            Keycode::Space => {
                // Space key can toggle pause/resume
                self.is_game_started = !self.is_game_started;
                println!("Game {}", if self.is_game_started { "resumed" } else { "paused" });
            }
            _ => {}
        }
        
        // Limit mover queue size like C++ version
        if self.mover.len() > 2 {
            self.mover.remove(1);
        }
    }

    pub fn update(&mut self) {
        if !self.is_game_started {
            return;
        }
        
        // Update Pacman position
        self.pacman.update_pos(&mut self.mover, &self.actual_map);
        
        // Check food collision and update score
        match self.pacman.food_collision(&mut self.actual_map) {
            0 => {
                // Pellet eaten
                self.board.score_increase(0);
                println!("Pellet eaten! Score: {}", self.board.get_score());
            }
            1 => {
                // Energizer eaten
                self.board.score_increase(1);
                self.pacman.change_energy_status(true);
                println!("Energizer eaten! Score: {}", self.board.get_score());
            }
            _ => {
                // No food eaten
            }
        }
    }

    pub fn draw(&mut self, canvas: &mut WindowCanvas, texture_creator: &'a TextureCreator<WindowContext>, font: &Font) -> Result<(), Box<dyn std::error::Error>> {
        // Update scores first (like C++ version - SetScore and SetHighScore called in Game::Draw)
        self.board.set_score(texture_creator, font)?;
        self.board.set_high_score(texture_creator, font)?;
        
        // Draw the board
        self.board.draw(canvas, &self.actual_map)?;

        // Draw Pacman
        self.pacman.draw(canvas)?;

        Ok(())
    }
}
