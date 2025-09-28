use crate::board::{BlockType, Board, Direction};
use crate::entity::{Blinky, Clyde, Entity, GhostBehavior, Inky, Pinky};
use crate::game_state::{GameState, GameTimer};
use crate::pacman::Pacman;
use crate::position::Position;
use crate::texture::GameTexture;
use crate::{BOARD_HEIGHT, BOARD_WIDTH, RED, WHITE, YELLOW};
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
    mover: Vec<Direction>,

    // NEW: Game state management (like C++ Game class)
    game_state: GameState,

    // NEW: Timer management (like C++ Timer GameTimer)
    game_timer: GameTimer,
    start_ticks: u32, // Time before game actually starts (2500ms)

    // NEW: Ghost timing (from C++ Game class)
    ghost_timer: GameTimer,
    scatter_time: u32,       // 7000ms - how long ghosts scatter
    chasing_time: u32,       // 20000ms - how long ghosts chase
    ghost_timer_target: u32, // Current target (chase or scatter)
    timed_status: bool,      // false = chase, true = scatter

    // NEW: Scoring system (like C++ Game)
    ghost_score_multiplier: u16, // 200, 400, 800, 1600
    dead_ghosts_counter: u8,

    // NEW: Animation and effects
    ready_texture: GameTexture<'a>,
    game_over_texture: GameTexture<'a>,
    little_score_timers: Vec<GameTimer>,
    little_score_positions: Vec<Position>,
    little_score_values: Vec<u16>,
    little_timer_target: u32, // 1000ms for floating score display

    // NEW: Level management
    level: u16,

    // NEW: Sound flags (like C++ Game)
    is_to_scatter_sound: bool,
    is_to_waka_sound: bool,
    is_to_death_sound: bool,
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

        // Create UI textures (like C++ constructor)
        let mut ready_texture = GameTexture::new();
        ready_texture.load_from_rendered_text(
            texture_creator,
            "READY!",
            &ttf_context.load_font("assets/emulogic.ttf", 24)?,
            YELLOW,
        )?;

        let mut game_over_texture = GameTexture::new();
        game_over_texture.load_from_rendered_text(
            texture_creator,
            "GAME  OVER",
            &ttf_context.load_font("assets/emulogic.ttf", 24)?,
            RED,
        )?;

        let mut game_timer = GameTimer::new();
        game_timer.start();

        Ok(Game {
            board,
            pacman,
            blinky,
            inky,
            pinky,
            clyde,
            actual_map,
            mover: vec![Direction::Right], // Default direction

            // Game state management (like C++ Game constructor)
            game_state: GameState::Ready,
            game_timer,
            start_ticks: 2500, // 2.5 seconds before game starts

            // Ghost timing (like C++ Game constructor)
            ghost_timer: GameTimer::new(),
            scatter_time: 7000,        // 7 seconds scatter
            chasing_time: 20000,       // 20 seconds chase
            ghost_timer_target: 20000, // Start with chasing
            timed_status: false,       // Start in chase mode

            // Scoring system
            ghost_score_multiplier: 200, // First ghost worth 200
            dead_ghosts_counter: 0,

            // Animation and effects
            ready_texture,
            game_over_texture,
            little_score_timers: Vec::new(),
            little_score_positions: Vec::new(),
            little_score_values: Vec::new(),
            little_timer_target: 1000, // 1 second for floating score

            // Level management
            level: 1,

            // Sound flags
            is_to_scatter_sound: true,
            is_to_waka_sound: true,
            is_to_death_sound: true,
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
                // Space key toggles pause/resume (like C++ version)
                match self.game_state {
                    GameState::Playing => {
                        self.game_state = GameState::Paused;
                        self.ghost_timer.pause();
                        println!("Game paused");
                    }
                    GameState::Paused => {
                        self.game_state = GameState::Playing;
                        self.ghost_timer.unpause();
                        println!("Game resumed");
                    }
                    _ => {
                        // Space can also start the game from Ready state
                        if self.game_state == GameState::Ready {
                            self.start_game();
                        }
                    }
                }
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
        // Use the new game state machine
        self.process();
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

        // Handle game state specific rendering
        match self.game_state {
            GameState::Ready => {
                // Show "READY!" text (like C++ Game::Start())
                self.ready_texture
                    .render(canvas, 11 * 24, 20 * 24 - 5, None)?;
            }
            GameState::GameOver => {
                // Show "GAME OVER" text (like C++ Game::Draw())
                self.game_over_texture
                    .render(canvas, 9 * 24, 20 * 24 - 5, None)?;
                return Ok(()); // Don't draw entities when game over
            }
            _ => {
                // Normal gameplay - draw entities
            }
        }

        // Draw entities (except during level complete animation)
        if self.game_state != GameState::LevelComplete {
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

            // Draw floating scores (like C++ Game::DrawLittleScore())
            self.draw_little_score();
        }

        // Draw Pacman on top
        self.pacman.draw(canvas)?;

        Ok(())
    }

    // NEW: Game state management methods (like C++ Game class)

    /// Start the game (like C++ Game::Start())
    fn start_game(&mut self) {
        if self.game_state == GameState::Ready {
            // Check if level is completed and need to reset map
            if self.is_level_completed() {
                self.board.copy_board(&mut self.actual_map);
            }

            // Positions are already reset when entering Ready state - no need to reset again

            // Reset game state (like C++ Game::Start())
            self.pacman.change_energy_status(false);
            self.reset_ghosts_life_statement();
            self.reset_ghosts_facing();
            self.pacman.reset_current_living_frame();

            // Start timers
            self.ghost_timer.restart();
            self.ghost_timer.start();

            self.game_state = GameState::Playing;
            println!("Game started!");
        }
    }

    /// Main game state machine (like C++ Game::Process())
    pub fn process(&mut self) -> bool {
        match self.game_state {
            GameState::Ready => {
                if self.game_timer.get_ticks() >= self.start_ticks as u128 {
                    self.start_game();
                }
            }
            GameState::Playing => {
                if self.pacman.is_alive() {
                    if !self.is_level_completed() {
                        self.update_game_logic();
                    } else {
                        // Level completed - start map animation
                        self.game_state = GameState::LevelComplete;
                        println!("Level {} completed!", self.level);
                    }
                } else {
                    // Pacman died
                    self.game_state = GameState::PacmanDeath;
                    println!("Pacman died!");
                }
            }
            GameState::PacmanDeath => {
                if self.pacman.is_dead_animation_ended() {
                    if self.board.get_lives() > 0 {
                        // Reset for new life
                        self.reset_game_for_death();

                        // Reset positions immediately when entering Ready state (like C++ version)
                        let pacman_start =
                            self.board.reset_position(crate::board::EntityType::PacMan);
                        self.pacman.set_position(pacman_start);

                        let blinky_start =
                            self.board.reset_position(crate::board::EntityType::Blinky);
                        self.blinky
                            .get_ghost_mut()
                            .entity
                            .set_position(blinky_start);

                        let inky_start = self.board.reset_position(crate::board::EntityType::Inky);
                        self.inky.get_ghost_mut().entity.set_position(inky_start);

                        let pinky_start =
                            self.board.reset_position(crate::board::EntityType::Pinky);
                        self.pinky.get_ghost_mut().entity.set_position(pinky_start);

                        let clyde_start =
                            self.board.reset_position(crate::board::EntityType::Clyde);
                        self.clyde.get_ghost_mut().entity.set_position(clyde_start);

                        self.game_state = GameState::Ready;
                        self.start_ticks = 2500;
                        self.game_timer.restart();
                    } else {
                        // Game over
                        self.game_state = GameState::GameOver;
                        println!("Game Over!");
                    }
                }
            }
            GameState::LevelComplete => {
                // TODO: Map flashing animation
                self.level += 1;
                self.update_difficulty();

                // Reset positions immediately when entering Ready state (like C++ version)
                let pacman_start = self.board.reset_position(crate::board::EntityType::PacMan);
                self.pacman.set_position(pacman_start);

                let blinky_start = self.board.reset_position(crate::board::EntityType::Blinky);
                self.blinky
                    .get_ghost_mut()
                    .entity
                    .set_position(blinky_start);

                let inky_start = self.board.reset_position(crate::board::EntityType::Inky);
                self.inky.get_ghost_mut().entity.set_position(inky_start);

                let pinky_start = self.board.reset_position(crate::board::EntityType::Pinky);
                self.pinky.get_ghost_mut().entity.set_position(pinky_start);

                let clyde_start = self.board.reset_position(crate::board::EntityType::Clyde);
                self.clyde.get_ghost_mut().entity.set_position(clyde_start);

                self.game_state = GameState::Ready;
                self.start_ticks = 2500;
                self.game_timer.restart();
                println!("Starting level {}", self.level);
            }
            GameState::GameOver => {
                // Game over state - wait for restart
            }
            GameState::Paused => {
                // Game is paused - do nothing
            }
        }

        true // Continue rendering
    }

    /// Update game logic when in Playing state (like C++ Game::Update())
    fn update_game_logic(&mut self) {
        self.clock(); // Manage ghost timing
        self.update_positions(); // Update all entity positions
        self.food_collision(); // Handle food eating
        self.entity_collisions(); // Handle all entity collisions
    }

    /// Manage ghost chase/scatter timing (like C++ Game::Clock())
    fn clock(&mut self) {
        if self.ghost_timer.get_ticks() > self.ghost_timer_target as u128 {
            if self.ghost_timer_target == self.scatter_time {
                // Switch from scatter to chase
                if self.pacman.is_energized() {
                    self.pacman.change_energy_status(false);
                }
                self.timed_status = false; // chase mode
                self.ghost_timer_target = self.chasing_time;
                self.ghost_timer.restart();
            } else if self.ghost_timer_target == self.chasing_time {
                // Switch from chase to scatter
                self.timed_status = true; // scatter mode
                self.ghost_timer_target = self.scatter_time;
                self.ghost_timer.restart();
            }
        }
    }

    /// Update all entity positions (like C++ Game::UpdatePositions())
    fn update_positions(&mut self) {
        // Update ghosts
        let pacman_pos = self.pacman.get_position();
        let blinky_pos = self.blinky.get_ghost().entity.get_position();

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

        // Update Pacman
        self.pacman.update_pos(&mut self.mover, &self.actual_map);
    }

    /// Handle food collision (like C++ Game::Food())
    fn food_collision(&mut self) {
        match self.pacman.food_collision(&mut self.actual_map) {
            0 => {
                // Pellet eaten
                self.board.score_increase(0);
                // TODO: Play waka sound
            }
            1 => {
                // Energizer eaten
                self.board.score_increase(1);
                self.pacman.change_energy_status(true);
                self.ghost_score_multiplier = 200; // Reset ghost score multiplier
                self.ghost_timer_target = self.scatter_time; // Force scatter mode
                self.ghost_timer.restart();
                // TODO: Play waka sound
            }
            _ => {
                // No food eaten
            }
        }
    }

    /// Handle all entity collisions (like C++ Game::EntityCollisions())
    fn entity_collisions(&mut self) {
        if !self.pacman.is_energized() {
            self.deadly_pac_ghost_collision();
            self.dead_ghosts_counter = 0; // Reset counter when not energized
        } else {
            self.deadly_ghost_pac_collision();
        }
    }

    /// Handle Pacman hitting live ghost (like C++ Game::DeadlyPacGhostColl())
    fn deadly_pac_ghost_collision(&mut self) {
        let pacman_pos = self.pacman.get_position();

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
        }
    }

    /// Handle energized Pacman eating ghosts (like C++ Game::DeadlyGhostPacColl())
    fn deadly_ghost_pac_collision(&mut self) {
        let pacman_pos = self.pacman.get_position();

        // Check each ghost and kill them inline to avoid borrowing issues
        if self
            .pacman
            .is_colliding(self.blinky.get_ghost().entity.get_position())
            && self.blinky.get_ghost().entity.is_alive()
        {
            self.blinky.get_ghost_mut().entity.mod_life_statement(false);
            self.board
                .score_increase_by_value(self.ghost_score_multiplier);

            // Add floating score animation (like C++ version)
            self.little_score_values.push(self.ghost_score_multiplier);
            let mut timer = GameTimer::new();
            timer.start();
            self.little_score_timers.push(timer);
            self.little_score_positions.push(pacman_pos);

            self.ghost_score_multiplier *= 2; // Double for next ghost
            self.dead_ghosts_counter += 1;
        }
        if self
            .pacman
            .is_colliding(self.inky.get_ghost().entity.get_position())
            && self.inky.get_ghost().entity.is_alive()
        {
            self.inky.get_ghost_mut().entity.mod_life_statement(false);
            self.board
                .score_increase_by_value(self.ghost_score_multiplier);

            // Add floating score animation
            self.little_score_values.push(self.ghost_score_multiplier);
            let mut timer = GameTimer::new();
            timer.start();
            self.little_score_timers.push(timer);
            self.little_score_positions.push(pacman_pos);

            self.ghost_score_multiplier *= 2;
            self.dead_ghosts_counter += 1;
        }
        if self
            .pacman
            .is_colliding(self.pinky.get_ghost().entity.get_position())
            && self.pinky.get_ghost().entity.is_alive()
        {
            self.pinky.get_ghost_mut().entity.mod_life_statement(false);
            self.board
                .score_increase_by_value(self.ghost_score_multiplier);

            // Add floating score animation
            self.little_score_values.push(self.ghost_score_multiplier);
            let mut timer = GameTimer::new();
            timer.start();
            self.little_score_timers.push(timer);
            self.little_score_positions.push(pacman_pos);

            self.ghost_score_multiplier *= 2;
            self.dead_ghosts_counter += 1;
        }
        if self
            .pacman
            .is_colliding(self.clyde.get_ghost().entity.get_position())
            && self.clyde.get_ghost().entity.is_alive()
        {
            self.clyde.get_ghost_mut().entity.mod_life_statement(false);
            self.board
                .score_increase_by_value(self.ghost_score_multiplier);

            // Add floating score animation
            self.little_score_values.push(self.ghost_score_multiplier);
            let mut timer = GameTimer::new();
            timer.start();
            self.little_score_timers.push(timer);
            self.little_score_positions.push(pacman_pos);

            self.ghost_score_multiplier *= 2;
            self.dead_ghosts_counter += 1;
        }
    }

    /// Reset ghosts life statements (like C++ Game::ResetGhostsLifeStatement())
    fn reset_ghosts_life_statement(&mut self) {
        self.blinky.get_ghost_mut().entity.mod_life_statement(true);
        self.inky.get_ghost_mut().entity.mod_life_statement(true);
        self.pinky.get_ghost_mut().entity.mod_life_statement(true);
        self.clyde.get_ghost_mut().entity.mod_life_statement(true);
    }

    /// Reset ghosts facing direction (like C++ Game::ResetGhostsFacing())
    fn reset_ghosts_facing(&mut self) {
        self.blinky
            .get_ghost_mut()
            .entity
            .set_facing(Direction::Left); // Blinky faces left
        self.inky.get_ghost_mut().entity.set_facing(Direction::Up); // Others face up
        self.pinky.get_ghost_mut().entity.set_facing(Direction::Up);
        self.clyde.get_ghost_mut().entity.set_facing(Direction::Up);
    }

    /// Check if level is completed (like C++ Game::IsLevelCompleted())
    fn is_level_completed(&self) -> bool {
        for &block in &self.actual_map {
            if block == BlockType::Pellet || block == BlockType::Energizer {
                return false;
            }
        }
        true
    }

    /// Reset game state after Pacman death (like C++ reset logic in Process())
    fn reset_game_for_death(&mut self) {
        self.clear_mover();
        self.pacman.mod_dead_animation_statement(false);
        self.pacman.mod_life_statement(true);
        self.board.decrease_lives();
        // TODO: Despawn fruit
        self.is_to_waka_sound = true;
        self.is_to_death_sound = true;
    }

    /// Clear movement queue (like C++ Game::ClearMover())
    fn clear_mover(&mut self) {
        self.mover.clear();
        self.mover.push(Direction::Right); // Default direction
    }

    /// Update difficulty for new level (like C++ Game::UpdateDifficulty())
    fn update_difficulty(&mut self) {
        if self.level % 3 == 0 {
            self.chasing_time += 1000; // Increase chase time by 1 second
            if self.scatter_time > 2000 {
                self.scatter_time -= 1000; // Decrease scatter time by 1 second
            }
        }
    }

    /// Draw floating scores (like C++ Game::DrawLittleScore())
    fn draw_little_score(&mut self) {
        // Remove expired timers
        let mut i = 0;
        while i < self.little_score_timers.len() {
            if self.little_score_timers[i].get_ticks() >= self.little_timer_target as u128 {
                self.little_score_timers.remove(i);
                self.little_score_positions.remove(i);
                self.little_score_values.remove(i);
            } else {
                i += 1;
            }
        }

        // TODO: Render remaining floating scores
    }
}
