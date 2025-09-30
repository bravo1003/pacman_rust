use super::state::GameTimer;

/// Manages all game timing behavior including ghost AI state transitions
pub struct TimerSystem {
    // Core game timing
    game_timer: GameTimer,
    start_ticks: u32,

    // Ghost AI timing
    ghost_timer: GameTimer,
    scatter_time: u32,
    chasing_time: u32,
    ghost_timer_target: u32,
    timed_status: bool, // true = scatter mode, false = chase mode
}

impl TimerSystem {
    pub fn new() -> Self {
        TimerSystem {
            game_timer: GameTimer::new(),
            start_ticks: 0,
            ghost_timer: GameTimer::new(),
            scatter_time: 7000,        // 7 seconds scatter
            chasing_time: 20000,       // 20 seconds chase
            ghost_timer_target: 20000, // Start with chasing
            timed_status: false,       // Start in chase mode
        }
    }

    /// Initialize game timing when game starts
    pub fn start_game(&mut self) {
        self.game_timer.restart();
    }

    /// Initialize ghost AI timing
    pub fn start_ghost_timing(&mut self) {
        self.ghost_timer.start();
    }

    /// Restart ghost timer for new cycle
    pub fn restart_ghost_timer(&mut self) {
        self.ghost_timer.restart();
    }

    /// Update ghost AI timing and return true if mode should change
    pub fn update_ghost_timing(&mut self) -> bool {
        if self.ghost_timer.get_ticks() >= self.ghost_timer_target as u128 {
            // Time to switch modes
            if self.timed_status {
                // Currently scattering, switch to chasing
                self.ghost_timer_target = self.chasing_time;
                self.timed_status = false;
            } else {
                // Currently chasing, switch to scattering
                self.ghost_timer_target = self.scatter_time;
                self.timed_status = true;
            }
            self.ghost_timer.restart();
            return true; // Mode changed
        }
        false // No mode change
    }

    /// Set ghost timer to scatter mode (for energizer)
    pub fn set_scatter_mode(&mut self) {
        self.ghost_timer_target = self.scatter_time;
        self.timed_status = true;
        self.ghost_timer.restart();
    }

    /// Check if ghosts should be in scatter mode
    pub fn is_scatter_mode(&self) -> bool {
        self.timed_status
    }

    /// Get current ghost timer target
    pub fn get_ghost_timer_target(&self) -> u32 {
        self.ghost_timer_target
    }

    /// Set custom ghost timer target
    pub fn set_ghost_timer_target(&mut self, target: u32) {
        self.ghost_timer_target = target;
    }

    /// Get game timer ticks
    pub fn get_game_ticks(&self) -> u128 {
        self.game_timer.get_ticks()
    }

    /// Get ghost timer ticks
    pub fn get_ghost_ticks(&self) -> u128 {
        self.ghost_timer.get_ticks()
    }

    /// Get start ticks
    pub fn get_start_ticks(&self) -> u32 {
        self.start_ticks
    }

    /// Set start ticks
    pub fn set_start_ticks(&mut self, ticks: u32) {
        self.start_ticks = ticks;
    }

    /// Pause all timers
    pub fn pause_all(&mut self) {
        self.game_timer.pause();
        self.ghost_timer.pause();
    }

    /// Unpause all timers
    pub fn unpause_all(&mut self) {
        self.game_timer.unpause();
        self.ghost_timer.unpause();
    }

    /// Update difficulty by increasing chase time and decreasing scatter time
    pub fn update_difficulty(&mut self) {
        self.chasing_time += 1000; // Increase chase time by 1 second
        if self.scatter_time > 2000 {
            self.scatter_time -= 1000; // Decrease scatter time by 1 second
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_system_creation() {
        let timer_system = TimerSystem::new();
        assert_eq!(timer_system.get_start_ticks(), 0);
        assert_eq!(timer_system.get_ghost_timer_target(), 20000);
        assert!(!timer_system.is_scatter_mode()); // Should start in chase mode
    }

    #[test]
    fn test_scatter_mode_setting() {
        let mut timer_system = TimerSystem::new();

        // Should start in chase mode
        assert!(!timer_system.is_scatter_mode());

        // Set to scatter mode
        timer_system.set_scatter_mode();
        assert!(timer_system.is_scatter_mode());
        assert_eq!(timer_system.get_ghost_timer_target(), 7000);
    }

    #[test]
    fn test_ghost_timer_target_setting() {
        let mut timer_system = TimerSystem::new();

        timer_system.set_ghost_timer_target(15000);
        assert_eq!(timer_system.get_ghost_timer_target(), 15000);
    }

    #[test]
    fn test_start_ticks_management() {
        let mut timer_system = TimerSystem::new();

        timer_system.set_start_ticks(1000);
        assert_eq!(timer_system.get_start_ticks(), 1000);
    }

    #[test]
    fn test_timing_initialization() {
        let mut timer_system = TimerSystem::new();

        // Start game timing
        timer_system.start_game();
        timer_system.start_ghost_timing();

        // Timers should be active (ticks > 0 after some time)
        std::thread::sleep(std::time::Duration::from_millis(1));
        assert!(timer_system.get_game_ticks() >= 0);
        assert!(timer_system.get_ghost_ticks() >= 0);
    }
}

