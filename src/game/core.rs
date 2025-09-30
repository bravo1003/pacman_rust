use super::collision::{CollisionEvent, CollisionSystem, GhostType};
use super::scoring::ScoringSystem;
use super::state::{GameState, GameTimer};
use super::timers::TimerSystem;
use crate::board::{BlockType, Board, Direction};
use crate::entity::pacman::Pacman;
use crate::entity::{Blinky, Clyde, Entity, GhostBehavior, Inky, Pinky};
use crate::texture::GameTexture;
use crate::{BOARD_HEIGHT, BOARD_WIDTH, RED, YELLOW};
use sdl2::keyboard::Keycode;
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::ttf::{Font, Sdl2TtfContext};
use sdl2::video::WindowContext;

pub struct Game<'a> {
    board: Board<'a>,
    pacman: Pacman<'a>,

    blinky: Blinky<'a>,
    inky: Inky<'a>,
    pinky: Pinky<'a>,
    clyde: Clyde<'a>,

    actual_map: [BlockType; BOARD_HEIGHT * BOARD_WIDTH],
    mover: Vec<Direction>,

    game_state: GameState,

    // Game systems
    timer_system: TimerSystem,
    collision_system: CollisionSystem,
    scoring_system: ScoringSystem,

    ready_texture: GameTexture<'a>,
    game_over_texture: GameTexture<'a>,
    paused_texture: GameTexture<'a>,

    level: u16,

    #[allow(dead_code)]
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

        let mut blinky = Blinky::new(texture_creator)?;
        let mut inky = Inky::new(texture_creator)?;
        let mut pinky = Pinky::new(texture_creator)?;
        let mut clyde = Clyde::new(texture_creator)?;

        let mut actual_map = [BlockType::Nothing; BOARD_HEIGHT * BOARD_WIDTH];
        board.copy_board(&mut actual_map);

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

        let font = ttf_context.load_font("assets/emulogic.ttf", 24)?;
        let mut ready_texture = GameTexture::new();
        ready_texture.load_from_rendered_text(texture_creator, "READY!", &font, YELLOW)?;

        let mut game_over_texture = GameTexture::new();
        game_over_texture.load_from_rendered_text(texture_creator, "GAME  OVER", &font, RED)?;

        let mut paused_texture = GameTexture::new();
        paused_texture.load_from_rendered_text(texture_creator, "PAUSED", &font, RED)?;

        let mut timer_system = TimerSystem::new();
        timer_system.set_start_ticks(2500); // 2.5 seconds before game starts
        timer_system.start_game();

        Ok(Game {
            board,
            pacman,
            blinky,
            inky,
            pinky,
            clyde,
            actual_map,
            mover: vec![Direction::Right],

            game_state: GameState::Ready,

            timer_system,

            collision_system: CollisionSystem::new(),
            scoring_system: ScoringSystem::new(),

            ready_texture,
            game_over_texture,
            paused_texture,

            level: 1,

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
            Keycode::Space => match self.game_state {
                GameState::Playing => {
                    self.game_state = GameState::Paused;
                    self.timer_system.pause_all();
                    println!("Game paused");
                }
                GameState::Paused => {
                    self.game_state = GameState::Playing;
                    self.timer_system.unpause_all();
                    println!("Game resumed");
                }
                _ => {
                    if self.game_state == GameState::Ready {
                        self.start_game();
                    }
                }
            },
            _ => {}
        }

        if self.mover.len() > 2 {
            self.mover.remove(1);
        }
    }

    pub fn update(&mut self) -> bool {
        match self.game_state {
            GameState::Ready => {
                if self.timer_system.get_game_ticks() >= self.timer_system.get_start_ticks() as u128
                {
                    self.start_game();
                }
            }
            GameState::Playing => {
                if self.pacman.is_alive() {
                    if !self.is_level_completed() {
                        self.update_game_logic();
                    } else {
                        self.game_state = GameState::LevelComplete;
                        println!("Level {} completed!", self.level);
                    }
                } else {
                    self.game_state = GameState::PacmanDeath;
                    println!("Pacman died!");
                }
            }
            GameState::PacmanDeath => {
                if self.pacman.is_dead_animation_ended() {
                    if self.board.get_lives() > 0 {
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
                        self.reset_game_for_death();
                    } else {
                        self.game_state = GameState::GameOver;
                        println!("Game Over!");
                    }
                }
            }
            GameState::LevelComplete => {
                // TODO: Map flashing animation
                self.level += 1;
                self.update_difficulty();

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
                self.timer_system.set_start_ticks(2500);
                self.timer_system.start_game();
                println!("Starting level {}", self.level);
            }
            GameState::GameOver => {}
            GameState::Paused => {}
        }

        true
    }

    pub fn draw(
        &mut self,
        canvas: &mut WindowCanvas,
        texture_creator: &'a TextureCreator<WindowContext>,
        font: &Font,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.board.set_score(texture_creator, font)?;
        self.board.set_high_score(texture_creator, font)?;

        self.board.draw(canvas, &self.actual_map)?;

        match self.game_state {
            GameState::Ready => {
                self.ready_texture
                    .render(canvas, 11 * 24, 20 * 24 - 5, None)?;
            }
            GameState::GameOver => {
                self.game_over_texture
                    .render(canvas, 9 * 24, 20 * 24 - 5, None)?;
                return Ok(());
            }
            GameState::Paused => {
                self.paused_texture
                    .render(canvas, 11 * 24, 20 * 24 - 5, None)?;
            }
            _ => {}
        }

        if self.game_state != GameState::LevelComplete {
            self.blinky.get_ghost_mut().draw(
                canvas,
                self.pacman.is_energized(),
                self.timer_system.get_ghost_ticks(),
                self.timer_system.get_ghost_timer_target(),
            )?;
            self.inky.get_ghost_mut().draw(
                canvas,
                self.pacman.is_energized(),
                self.timer_system.get_ghost_ticks(),
                self.timer_system.get_ghost_timer_target(),
            )?;
            self.pinky.get_ghost_mut().draw(
                canvas,
                self.pacman.is_energized(),
                self.timer_system.get_ghost_ticks(),
                self.timer_system.get_ghost_timer_target(),
            )?;
            self.clyde.get_ghost_mut().draw(
                canvas,
                self.pacman.is_energized(),
                self.timer_system.get_ghost_ticks(),
                self.timer_system.get_ghost_timer_target(),
            )?;

            self.draw_little_score();
        }

        self.pacman.draw(canvas)?;

        Ok(())
    }

    fn start_game(&mut self) {
        if self.game_state == GameState::Ready {
            if self.is_level_completed() {
                self.board.copy_board(&mut self.actual_map);
            }

            self.pacman.change_energy_status(false);
            self.reset_ghosts_life_statement();
            self.reset_ghosts_facing();
            self.pacman.reset_current_living_frame();

            self.timer_system.restart_ghost_timer();
            self.timer_system.start_ghost_timing();

            self.game_state = GameState::Playing;
            println!("Game started!");
        }
    }

    fn update_game_logic(&mut self) {
        self.clock();
        self.update_positions();
        self.food_collision();
        self.entity_collisions();
    }

    fn clock(&mut self) {
        if self.timer_system.update_ghost_timing() {
            // Ghost mode changed, check if we need to end energizer
            if !self.timer_system.is_scatter_mode() && self.pacman.is_energized() {
                self.pacman.change_energy_status(false);
            }
        }
    }

    fn update_positions(&mut self) {
        let blinky_pos = self.blinky.get_ghost().entity.get_position();

        self.blinky.update_pos(
            &self.actual_map,
            &self.pacman,
            None,
            self.timer_system.is_scatter_mode(),
        );
        self.inky.update_pos(
            &self.actual_map,
            &self.pacman,
            Some(blinky_pos),
            self.timer_system.is_scatter_mode(),
        );
        self.pinky.update_pos(
            &self.actual_map,
            &self.pacman,
            None,
            self.timer_system.is_scatter_mode(),
        );
        self.clyde.update_pos(
            &self.actual_map,
            &self.pacman,
            None,
            self.timer_system.is_scatter_mode(),
        );

        self.pacman.update_pos(&mut self.mover, &self.actual_map);
    }

    fn food_collision(&mut self) {
        match self.pacman.food_collision(&mut self.actual_map) {
            0 => {
                self.board.score_increase(0);
                // TODO: Play waka sound
            }
            1 => {
                self.board.score_increase(1);
                self.pacman.change_energy_status(true);
                self.scoring_system.reset_for_energizer();
                self.timer_system.set_scatter_mode();
                // TODO: Play waka sound
            }
            _ => {}
        }
    }

    fn entity_collisions(&mut self) {
        if !self.pacman.is_energized() {
            self.scoring_system.reset_ghost_counter();
        }
        self.check_ghost_collisions();
    }

    fn check_ghost_collisions(&mut self) {
        let collisions = self.collision_system.check_all_ghost_collisions(
            &self.pacman,
            &self.blinky,
            &self.inky,
            &self.pinky,
            &self.clyde,
            self.pacman.is_energized(),
        );

        for collision in collisions {
            match collision {
                CollisionEvent::PacmanEatsGhost {
                    ghost_type,
                    position,
                } => {
                    // Handle Pacman eating a ghost
                    match ghost_type {
                        GhostType::Blinky => {
                            self.blinky.get_ghost_mut().entity.mod_life_statement(false);
                        }
                        GhostType::Inky => {
                            self.inky.get_ghost_mut().entity.mod_life_statement(false);
                        }
                        GhostType::Pinky => {
                            self.pinky.get_ghost_mut().entity.mod_life_statement(false);
                        }
                        GhostType::Clyde => {
                            self.clyde.get_ghost_mut().entity.mod_life_statement(false);
                        }
                    }

                    // Award points and add floating score
                    let score_value = self.scoring_system.add_ghost_score(position);
                    self.board.score_increase_by_value(score_value);
                }
                CollisionEvent::GhostKillsPacman { ghost_type: _ } => {
                    // Handle ghost killing Pacman
                    self.pacman.mod_life_statement(false);
                    // Only need to handle one death, so break after first
                    break;
                }
                CollisionEvent::NoCollision => {
                    // This shouldn't happen since we filter out NoCollision events
                }
            }
        }
    }

    fn reset_ghosts_life_statement(&mut self) {
        self.blinky.get_ghost_mut().entity.mod_life_statement(true);
        self.inky.get_ghost_mut().entity.mod_life_statement(true);
        self.pinky.get_ghost_mut().entity.mod_life_statement(true);
        self.clyde.get_ghost_mut().entity.mod_life_statement(true);
    }

    fn reset_ghosts_facing(&mut self) {
        self.blinky
            .get_ghost_mut()
            .entity
            .set_facing(Direction::Left);
        self.inky.get_ghost_mut().entity.set_facing(Direction::Up);
        self.pinky
            .get_ghost_mut()
            .entity
            .set_facing(Direction::Down);
        self.clyde.get_ghost_mut().entity.set_facing(Direction::Up);
    }

    fn is_level_completed(&self) -> bool {
        for &block in &self.actual_map {
            if block == BlockType::Pellet || block == BlockType::Energizer {
                return false;
            }
        }
        true
    }

    fn reset_game_for_death(&mut self) {
        self.clear_mover();
        self.pacman.mod_dead_animation_statement(false);
        self.pacman.mod_life_statement(true);
        self.pacman.change_energy_status(false);
        self.pacman.reset_current_living_frame();
        self.board.decrease_lives();

        self.reset_ghosts_life_statement();
        self.reset_ghosts_facing();

        // TODO: Despawn fruit
        self.is_to_waka_sound = true;
        self.is_to_death_sound = true;

        // Reset ghost timer and start ghost timing
        self.timer_system.restart_ghost_timer();
        self.timer_system.start_ghost_timing();

        // Reset game timer for 2.5 second delay - order is important!
        self.timer_system.set_start_ticks(2500);
        self.timer_system.start_game();
    }

    fn clear_mover(&mut self) {
        self.mover.clear();
        self.mover.push(Direction::Right);
    }

    fn update_difficulty(&mut self) {
        if self.level.is_multiple_of(3) {
            self.timer_system.update_difficulty();
        }
    }

    fn draw_little_score(&mut self) {
        self.scoring_system.update_little_scores();
        // TODO: Render remaining floating scores using self.scoring_system.get_little_scores()
    }
}
