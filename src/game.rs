use crate::board::{BlockType, Board, Direction};
use crate::entity::{Blinky, Clyde, Entity, GhostBehavior, Inky, Pinky};
use crate::pacman::Pacman;
use crate::{BOARD_HEIGHT, BOARD_WIDTH};
use sdl2::keyboard::Keycode;
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::ttf::{Font, Sdl2TtfContext};
use sdl2::video::WindowContext;

pub struct Game<'a> {
    board: Board<'a>,
    pacman: Pacman<'a>,

    // The 4 ghosts with individual personalities
    blinky: Blinky<'a>,
    inky: Inky<'a>,
    pinky: Pinky<'a>,
    clyde: Clyde<'a>,

    actual_map: [BlockType; BOARD_HEIGHT * BOARD_WIDTH],
    is_game_started: bool,
    mover: Vec<Direction>,

    // Ghost timing (like C++ version)
    timed_status: bool, // false = chase, true = scatter
}

impl<'a> Game<'a> {
    pub fn new(
        texture_creator: &'a TextureCreator<WindowContext>,
        ttf_context: &'a Sdl2TtfContext,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let board = Board::new(texture_creator, ttf_context)?;
        let mut pacman = Pacman::new(texture_creator)?;

        // Create the 4 ghosts with individual personalities
        let mut blinky = Blinky::new(texture_creator)?;
        let mut inky = Inky::new(texture_creator)?;
        let mut pinky = Pinky::new(texture_creator)?;
        let mut clyde = Clyde::new(texture_creator)?;

        let mut actual_map = [BlockType::Nothing; BOARD_HEIGHT * BOARD_WIDTH];
        board.copy_board(&mut actual_map);

        // Position entities at their starting positions (like C++ ResetPosition)
        let pacman_start = board.reset_position(crate::board::EntityType::PacMan);
        pacman.set_position(pacman_start);

        let blinky_start = board.reset_position(crate::board::EntityType::Blinky);
        blinky.get_ghost_mut().entity.set_position(blinky_start);

        let inky_start = board.reset_position(crate::board::EntityType::Inky);
        inky.get_ghost_mut().entity.set_position(inky_start);

        let pinky_start = board.reset_position(crate::board::EntityType::Pinky);
        pinky.get_ghost_mut().entity.set_position(pinky_start);

        let clyde_start = board.reset_position(crate::board::EntityType::Clyde);
        clyde.get_ghost_mut().entity.set_position(clyde_start);

        Ok(Game {
            board,
            pacman,
            blinky,
            inky,
            pinky,
            clyde,
            actual_map,
            is_game_started: true, // Start immediately for easier debugging
            mover: vec![Direction::Right], // Default direction
            timed_status: false,   // Start in chase mode
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
                println!(
                    "Game {}",
                    if self.is_game_started {
                        "resumed"
                    } else {
                        "paused"
                    }
                );
            }
            _ => {}
        }

        // Limit mover queue size like C++ version
        if self.mover.len() > 2 {
            self.mover.remove(1);
        }
    }

    // Individual ghost update methods to avoid borrowing issues
    fn update_blinky(&mut self) {
        let ghost = self.blinky.get_ghost_mut();
        ghost.update_status(self.timed_status);

        for _ in 0..ghost.entity.get_speed() {
            ghost.update_facing(self.pacman.is_energized());
            
            // Check if we should use normal AI target (or home exit logic)
            if ghost.should_calculate_normal_target(self.pacman.is_energized()) {
                // Normal AI targeting already set by calculate_target
            }
            
            ghost.calculate_direction(&self.actual_map);
            ghost.entity.move_entity(ghost.entity.get_direction());
            ghost.entity.check_wrap();
        }
    }

    fn update_inky(&mut self) {
        let ghost = self.inky.get_ghost_mut();
        ghost.update_status(self.timed_status);

        for _ in 0..ghost.entity.get_speed() {
            ghost.update_facing(self.pacman.is_energized());
            
            // Check if we should use normal AI target (or home exit logic)
            if ghost.should_calculate_normal_target(self.pacman.is_energized()) {
                // Normal AI targeting already set by calculate_target
            }
            
            ghost.calculate_direction(&self.actual_map);
            ghost.entity.move_entity(ghost.entity.get_direction());
            ghost.entity.check_wrap();
        }
    }

    fn update_pinky(&mut self) {
        let ghost = self.pinky.get_ghost_mut();
        ghost.update_status(self.timed_status);

        for _ in 0..ghost.entity.get_speed() {
            ghost.update_facing(self.pacman.is_energized());
            
            // Check if we should use normal AI target (or home exit logic)
            if ghost.should_calculate_normal_target(self.pacman.is_energized()) {
                // Normal AI targeting already set by calculate_target
            }
            
            ghost.calculate_direction(&self.actual_map);
            ghost.entity.move_entity(ghost.entity.get_direction());
            ghost.entity.check_wrap();
        }
    }

    fn update_clyde(&mut self) {
        let ghost = self.clyde.get_ghost_mut();
        ghost.update_status(self.timed_status);

        for _ in 0..ghost.entity.get_speed() {
            ghost.update_facing(self.pacman.is_energized());
            
            // Check if we should use normal AI target (or home exit logic)
            if ghost.should_calculate_normal_target(self.pacman.is_energized()) {
                // Normal AI targeting already set by calculate_target
            }
            
            ghost.calculate_direction(&self.actual_map);
            ghost.entity.move_entity(ghost.entity.get_direction());
            ghost.entity.check_wrap();
        }
    }

    // Check ghost-Pacman collisions (like C++ version)
    fn check_ghost_collisions(&mut self) {
        if !self.pacman.is_energized() {
            // Normal mode - ghosts can kill Pacman
            if (self
                .pacman
                .is_colliding(self.blinky.get_ghost().entity.get_position())
                && self.blinky.get_ghost().entity.is_alive())
                || (self
                    .pacman
                    .is_colliding(self.inky.get_ghost().entity.get_position())
                    && self.inky.get_ghost().entity.is_alive())
                || (self
                    .pacman
                    .is_colliding(self.pinky.get_ghost().entity.get_position())
                    && self.pinky.get_ghost().entity.is_alive())
                || (self
                    .pacman
                    .is_colliding(self.clyde.get_ghost().entity.get_position())
                    && self.clyde.get_ghost().entity.is_alive())
            {
                self.pacman.mod_life_statement(false);
                println!("Pacman caught by ghost!");
            }
        } else {
            // Energized mode - Pacman can eat ghosts
            if self
                .pacman
                .is_colliding(self.blinky.get_ghost().entity.get_position())
                && self.blinky.get_ghost().entity.is_alive()
            {
                self.blinky.get_ghost_mut().entity.mod_life_statement(false);
                self.board.score_increase(2); // Ghost points
                println!("Blinky eaten! Score: {}", self.board.get_score());
            }
            if self
                .pacman
                .is_colliding(self.inky.get_ghost().entity.get_position())
                && self.inky.get_ghost().entity.is_alive()
            {
                self.inky.get_ghost_mut().entity.mod_life_statement(false);
                self.board.score_increase(2); // Ghost points
                println!("Inky eaten! Score: {}", self.board.get_score());
            }
            if self
                .pacman
                .is_colliding(self.pinky.get_ghost().entity.get_position())
                && self.pinky.get_ghost().entity.is_alive()
            {
                self.pinky.get_ghost_mut().entity.mod_life_statement(false);
                self.board.score_increase(2); // Ghost points
                println!("Pinky eaten! Score: {}", self.board.get_score());
            }
            if self
                .pacman
                .is_colliding(self.clyde.get_ghost().entity.get_position())
                && self.clyde.get_ghost().entity.is_alive()
            {
                self.clyde.get_ghost_mut().entity.mod_life_statement(false);
                self.board.score_increase(2); // Ghost points
                println!("Clyde eaten! Score: {}", self.board.get_score());
            }
        }
    }

    pub fn update(&mut self) {
        // Only update game logic if the game is running
        if !self.is_game_started {
            return;
        }

        // Update Pacman position
        self.pacman.update_pos(&mut self.mover, &self.actual_map);

        // Update ghosts with their individual AI personalities
        let blinky_pos = self.blinky.get_ghost().entity.get_position();
        let pacman_pos = self.pacman.get_position();

        // Update Blinky
        self.blinky.calculate_target(pacman_pos, None);
        self.update_blinky();

        // Update Inky (needs Blinky position)
        self.inky.calculate_target(pacman_pos, Some(blinky_pos));
        self.update_inky();

        // Update Pinky
        self.pinky.calculate_target(pacman_pos, None);
        self.update_pinky();

        // Update Clyde
        self.clyde.calculate_target(pacman_pos, None);
        self.update_clyde();

        // Check food collision and update score

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

        // Check ghost-Pacman collisions
        self.check_ghost_collisions();
    }

    pub fn draw(
        &mut self,
        canvas: &mut WindowCanvas,
        texture_creator: &'a TextureCreator<WindowContext>,
        font: &Font,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Update scores first (like C++ version - SetScore and SetHighScore called in Game::Draw)
        self.board.set_score(texture_creator, font)?;
        self.board.set_high_score(texture_creator, font)?;

        // Draw the board
        self.board.draw(canvas, &self.actual_map)?;

        // Draw ghosts first (like C++ version - ghosts drawn before Pacman)
        self.blinky
            .get_ghost_mut()
            .draw(canvas, self.pacman.is_energized())?;
        self.inky
            .get_ghost_mut()
            .draw(canvas, self.pacman.is_energized())?;
        self.pinky
            .get_ghost_mut()
            .draw(canvas, self.pacman.is_energized())?;
        self.clyde
            .get_ghost_mut()
            .draw(canvas, self.pacman.is_energized())?;

        // Draw Pacman on top
        self.pacman.draw(canvas)?;

        Ok(())
    }
}
