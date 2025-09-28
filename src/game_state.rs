use std::time::Instant;

#[derive(Debug, Clone, PartialEq)]
pub enum GameState {
    Ready,        // Show "READY!" text before game starts
    Playing,      // Normal gameplay
    PacmanDeath,  // Pacman death animation
    GameOver,     // All lives lost
    LevelComplete,// All pellets eaten - map flashing animation
    Paused,       // Game paused with space
}

/// Timer utility similar to C++ Timer class
#[derive(Debug, Clone)]
pub struct GameTimer {
    start_time: Option<Instant>,
    is_paused: bool,
    pause_time: Option<Instant>,
    accumulated_time: u128,
}

impl GameTimer {
    pub fn new() -> Self {
        GameTimer {
            start_time: None,
            is_paused: false,
            pause_time: None,
            accumulated_time: 0,
        }
    }

    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
        self.is_paused = false;
        self.pause_time = None;
    }

    pub fn restart(&mut self) {
        self.start_time = Some(Instant::now());
        self.accumulated_time = 0;
        self.is_paused = false;
        self.pause_time = None;
    }

    pub fn reset(&mut self) {
        self.start_time = None;
        self.is_paused = false;
        self.pause_time = None;
        self.accumulated_time = 0;
    }

    pub fn pause(&mut self) {
        if !self.is_paused && self.start_time.is_some() {
            self.pause_time = Some(Instant::now());
            self.is_paused = true;
        }
    }

    pub fn unpause(&mut self) {
        if self.is_paused {
            if let Some(pause_time) = self.pause_time {
                self.accumulated_time += pause_time.elapsed().as_millis();
            }
            self.is_paused = false;
            self.pause_time = None;
            self.start_time = Some(Instant::now());
        }
    }

    pub fn get_ticks(&self) -> u128 {
        if let Some(start) = self.start_time {
            if self.is_paused {
                if let Some(pause_time) = self.pause_time {
                    return self.accumulated_time + start.elapsed().as_millis() - pause_time.elapsed().as_millis();
                }
            }
            return self.accumulated_time + start.elapsed().as_millis();
        }
        0
    }

    pub fn is_started(&self) -> bool {
        self.start_time.is_some()
    }
}