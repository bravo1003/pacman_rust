use std::path::PathBuf;

use sdl2::pixels::Color;

use crate::audio::{Sound, SoundCommand};
use crate::direction::Direction;
use crate::fruit::{self, Fruit};
use crate::geometry::{Point, SPRITE, TILE, WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::gfx::{self, Counter, Gfx, Image};
use crate::ghost::{Ghost, GhostKind};
use crate::high_score::HighScore;
use crate::maze::{Food, Maze};
use crate::pacman::{Pacman, Steering};
use crate::timer::Timer;

/// How long "ready!" shows before the first round, and before later ones.
const FIRST_ROUND_DELAY_MS: u32 = 4500;
const ROUND_DELAY_MS: u32 = 2500;
/// A cleared maze flashes for this long before the next level.
const LEVEL_END_MS: u32 = 2100;
const MAZE_FLASH_PERIOD_MS: u32 = 250;
/// The waka sound stops once Pac-Man has gone this long without eating.
const WAKA_SILENCE_MS: u32 = 300;
const POPUP_MS: u32 = 1000;
const PELLET_POINTS: u32 = 10;
const ENERGIZER_POINTS: u32 = 50;
const FIRST_GHOST_POINTS: u32 = 200;
const EXTRA_LIFE_SCORE: u32 = 10_000;

/// Points shown where a ghost was eaten.
#[derive(Clone, Copy)]
struct Popup {
    points: u32,
    at: Point,
    shown: Timer,
}

pub struct Game {
    maze: Maze,
    pacman: Pacman,
    /// In update order: Blinky, Inky, Pinky, Clyde.
    ghosts: [Ghost; 4],
    fruit: Fruit,
    popups: Vec<Popup>,
    steering: Steering,
    sounds: Vec<SoundCommand>,
    high_score: HighScore,
    score: u32,
    lives: u32,
    extra_life_awarded: bool,
    level: u32,
    /// Whether a round is being played; false between rounds and once the
    /// game is over.
    playing: bool,
    showing_ready: bool,
    /// Restarted for each round; the round begins `round_delay` after it.
    round_timer: Timer,
    round_delay: u32,
    level_end_timer: Timer,
    maze_tint: Color,
    /// Times the current scatter, chase or fright phase.
    phase_timer: Timer,
    phase_length: u32,
    scatter_time: u32,
    chase_time: u32,
    scatter_phase: bool,
    ghost_points: u32,
    ghosts_eaten: u32,
    fright_sound_on: bool,
    waka_on: bool,
    waka_timer: Timer,
    death_sound_played: bool,
}

impl Game {
    /// Starts a new game at time `now`, in milliseconds.
    pub fn new(now: u32, high_score_path: impl Into<PathBuf>) -> Self {
        let chase_time = 20_000;
        Self {
            maze: Maze::new(),
            pacman: Pacman::new(),
            ghosts: [
                Ghost::new(GhostKind::Blinky),
                Ghost::new(GhostKind::Inky),
                Ghost::new(GhostKind::Pinky),
                Ghost::new(GhostKind::Clyde),
            ],
            fruit: Fruit::new(),
            popups: Vec::new(),
            steering: Steering::new(),
            sounds: vec![SoundCommand::Play(Sound::Intro)],
            high_score: HighScore::load(high_score_path.into()),
            score: 0,
            lives: 4,
            extra_life_awarded: false,
            level: 1,
            playing: false,
            showing_ready: false,
            round_timer: Timer::started(now),
            round_delay: FIRST_ROUND_DELAY_MS,
            level_end_timer: Timer::default(),
            maze_tint: gfx::BLUE,
            phase_timer: Timer::default(),
            phase_length: chase_time,
            scatter_time: 7000,
            chase_time,
            scatter_phase: false,
            ghost_points: FIRST_GHOST_POINTS,
            ghosts_eaten: 0,
            fright_sound_on: false,
            waka_on: false,
            waka_timer: Timer::default(),
            death_sound_played: false,
        }
    }

    pub fn steer(&mut self, direction: Direction) {
        self.steering.request(direction);
    }

    /// Sound effects requested since the last call.
    pub fn drain_sounds(&mut self) -> impl Iterator<Item = SoundCommand> + '_ {
        self.sounds.drain(..)
    }

    /// Advances the game by one frame. Returns false on frames that must not
    /// be drawn: the original skips the frame on which a new round is set up.
    pub fn update(&mut self, now: u32) -> bool {
        self.showing_ready = self.round_timer.ticks(now) < self.round_delay;
        if self.showing_ready {
            self.start_round(now);
            true
        } else if !self.pacman.alive {
            self.update_death(now)
        } else if self.maze.has_food() {
            self.play_frame(now);
            true
        } else {
            self.update_level_end(now)
        }
    }

    /// Waits out the death animation, then starts the next round or ends
    /// the game.
    fn update_death(&mut self, now: u32) -> bool {
        if self.pacman.death_animation_done() {
            if self.lives == 0 {
                self.playing = false;
            } else {
                self.lives -= 1;
                self.pacman.revive();
                self.fruit.despawn();
                self.waka_on = false;
                self.death_sound_played = false;
                self.next_round(now);
                return false;
            }
        }
        if !self.death_sound_played {
            self.stop(Sound::Waka);
            self.play(Sound::PacDeath);
            self.death_sound_played = true;
        }
        true
    }

    /// Flashes the cleared maze, then moves on to the next level.
    fn update_level_end(&mut self, now: u32) -> bool {
        if !self.level_end_timer.is_started() {
            self.pacman.reset_mouth();
            self.fruit.despawn();
            self.fruit.reset_food_count();
            self.stop(Sound::Waka);
            self.stop(Sound::ScatterGhost);
            self.waka_on = false;
            self.level_end_timer.start(now);
        } else if self.level_end_timer.ticks(now) > LEVEL_END_MS {
            self.level_end_timer.reset();
            self.level += 1;
            self.fruit.set_level(self.level);
            self.raise_difficulty();
            self.next_round(now);
            return false;
        }
        true
    }

    /// Shows "ready!" again before the next round starts.
    fn next_round(&mut self, now: u32) {
        self.steering.reset();
        self.playing = false;
        self.round_delay = ROUND_DELAY_MS;
        self.round_timer.start(now);
    }

    /// Puts everyone in place for a new round, once per round.
    fn start_round(&mut self, now: u32) {
        if self.playing {
            return;
        }
        if !self.maze.has_food() {
            self.maze = Maze::new();
        }
        self.pacman.respawn();
        for ghost in &mut self.ghosts {
            ghost.respawn();
        }
        self.phase_timer.start(now);
        self.playing = true;
    }

    fn play_frame(&mut self, now: u32) {
        self.advance_phase(now);
        for i in 0..self.ghosts.len() {
            let blinky = self.ghosts[0].pos;
            self.ghosts[i].update(&self.maze, &self.pacman, blinky, self.scatter_phase);
        }
        self.pacman.update(&mut self.steering, &self.maze);
        self.eat_food(now);
        if !self.extra_life_awarded && self.score >= EXTRA_LIFE_SCORE {
            self.extra_life_awarded = true;
            self.lives += 1;
            self.play(Sound::ExtraLife);
        }
        self.resolve_collisions(now);
        self.high_score.record(self.score);
    }

    /// Alternates the ghosts between chasing and scattering; the end of a
    /// scatter phase also ends Pac-Man's fright.
    fn advance_phase(&mut self, now: u32) {
        if self.phase_timer.ticks(now) <= self.phase_length {
            return;
        }
        // A phase length left over from before a difficulty change matches
        // neither phase, which stalls the cycle until the next energizer,
        // exactly as in the original.
        if self.phase_length == self.scatter_time {
            self.pacman.energized = false;
            self.scatter_phase = false;
            self.phase_length = self.chase_time;
            self.phase_timer.start(now);
        } else if self.phase_length == self.chase_time {
            self.scatter_phase = true;
            self.phase_length = self.scatter_time;
            self.phase_timer.start(now);
        }
    }

    fn eat_food(&mut self, now: u32) {
        match self.maze.eat(self.pacman.pos) {
            Some(food) => {
                self.fruit.food_eaten(now);
                match food {
                    Food::Pellet => self.score += PELLET_POINTS,
                    Food::Energizer => {
                        self.score += ENERGIZER_POINTS;
                        self.pacman.energized = true;
                        self.ghost_points = FIRST_GHOST_POINTS;
                        self.phase_length = self.scatter_time;
                        self.phase_timer.start(now);
                    }
                }
                if !self.waka_on {
                    self.play(Sound::Waka);
                    self.waka_on = true;
                }
                self.waka_timer.reset();
            }
            None if !self.waka_timer.is_started() => self.waka_timer.start(now),
            None => {}
        }
        if self.waka_timer.ticks(now) > WAKA_SILENCE_MS {
            self.stop(Sound::Waka);
            self.waka_on = false;
        }
    }

    fn resolve_collisions(&mut self, now: u32) {
        if self.pacman.energized {
            if !self.fright_sound_on {
                self.play(Sound::ScatterGhost);
                self.fright_sound_on = true;
            }
            for i in 0..self.ghosts.len() {
                self.try_eat_ghost(i, now);
            }
            if self.ghosts_eaten == 4 && self.fright_sound_on {
                self.stop(Sound::ScatterGhost);
            }
        } else {
            let pacman = self.pacman.pos;
            if self
                .ghosts
                .iter()
                .any(|ghost| ghost.alive && pacman.overlaps(ghost.pos))
            {
                self.pacman.alive = false;
            }
            if self.fright_sound_on {
                self.ghosts_eaten = 0;
                self.stop(Sound::ScatterGhost);
                self.fright_sound_on = false;
            }
        }
        if self.fruit.is_shown() {
            if self.pacman.pos.overlaps(fruit::POSITION) {
                self.fruit.eat(now);
                self.score += self.fruit.points();
                self.play(Sound::EatFruit);
            } else if self.fruit.has_expired(now) {
                self.fruit.despawn();
            }
        }
    }

    fn try_eat_ghost(&mut self, index: usize, now: u32) {
        let ghost = &mut self.ghosts[index];
        if !ghost.alive || !self.pacman.pos.overlaps(ghost.pos) {
            return;
        }
        ghost.alive = false;
        self.score += self.ghost_points;
        self.popups.push(Popup {
            points: self.ghost_points,
            at: ghost.pos,
            shown: Timer::started(now),
        });
        self.ghost_points *= 2;
        self.play(Sound::GhostDeath);
        self.ghosts_eaten += 1;
    }

    /// Every third level, ghosts chase longer and scatter less.
    fn raise_difficulty(&mut self) {
        if self.level.is_multiple_of(3) {
            self.chase_time += 1000;
            if self.scatter_time > 2000 {
                self.scatter_time -= 1000;
            }
        }
    }

    fn play(&mut self, sound: Sound) {
        self.sounds.push(SoundCommand::Play(sound));
    }

    fn stop(&mut self, sound: Sound) {
        self.sounds.push(SoundCommand::Stop(sound));
    }

    /// Draws the frame; this also advances the sprite animations.
    pub fn draw(&mut self, gfx: &mut Gfx, now: u32) -> Result<(), String> {
        if self.showing_ready {
            gfx.image(Image::Ready, Point::new(11 * TILE, 20 * TILE - 5))?;
        }
        self.draw_board(gfx, now)?;
        if !self.playing {
            return gfx.image(Image::GameOver, Point::new(9 * TILE, 20 * TILE - 5));
        }
        self.fruit.draw(gfx, now)?;
        if !self.level_end_timer.is_started() {
            let phase_ms = self.phase_timer.ticks(now);
            for ghost in self.ghosts.iter_mut().rev() {
                ghost.draw(gfx, self.pacman.energized, phase_ms, self.scatter_time)?;
            }
            self.draw_popups(gfx, now)?;
        }
        self.pacman.draw(gfx)
    }

    fn draw_board(&mut self, gfx: &mut Gfx, now: u32) -> Result<(), String> {
        let right_column = WINDOW_WIDTH / 2;
        gfx.image(Image::ScoreLabel, Point::new(0, 0))?;
        gfx.counter(Counter::Score, self.score, Point::new(0, SPRITE))?;
        gfx.image(Image::HighScoreLabel, Point::new(right_column, 0))?;
        gfx.counter(
            Counter::HighScore,
            self.high_score.best(),
            Point::new(right_column, SPRITE),
        )?;
        gfx.tinted_image(Image::Map, Point::new(0, 0), self.maze_tint)?;
        for life in 1..=self.lives as i32 {
            gfx.image(
                Image::Life,
                Point::new(life * SPRITE, 26 * SPRITE - SPRITE / 4),
            )?;
        }
        if self.level_end_timer.is_started() {
            // Takes effect on the next frame, as in the original.
            let flash = self.level_end_timer.ticks(now) / MAZE_FLASH_PERIOD_MS % 2 == 1;
            self.maze_tint = if flash { gfx::WHITE } else { gfx::BLUE };
            return Ok(());
        }
        let door = Point::new(WINDOW_WIDTH / 2 - 23, WINDOW_HEIGHT / 2 - 57);
        gfx.image(Image::Door, door)?;
        for (at, food) in self.maze.food() {
            let image = match food {
                Food::Pellet => Image::Pellet,
                Food::Energizer => Image::Energizer,
            };
            gfx.image(image, at)?;
        }
        Ok(())
    }

    fn draw_popups(&mut self, gfx: &mut Gfx, now: u32) -> Result<(), String> {
        let mut i = 0;
        while i < self.popups.len() {
            let popup = self.popups[i];
            if popup.shown.ticks(now) < POPUP_MS {
                gfx.small_number(popup.points, Point::new(popup.at.x, popup.at.y - TILE / 2))?;
            } else {
                self.popups.remove(i);
            }
            // Advancing past a removal skips the next popup for one frame,
            // matching the original.
            i += 1;
        }
        Ok(())
    }
}
