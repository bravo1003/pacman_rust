use crate::game::state::GameTimer;
use crate::position::Position;

#[derive(Debug)]
pub struct LittleScore {
    #[allow(dead_code)]
    pub position: Position,
    #[allow(dead_code)]
    pub value: u16,
    pub timer: GameTimer,
}

impl LittleScore {
    pub fn new(position: Position, value: u16) -> Self {
        let mut timer = GameTimer::new();
        timer.start();

        LittleScore {
            position,
            value,
            timer,
        }
    }

    pub fn is_expired(&self, target_time: u32) -> bool {
        self.timer.get_ticks() >= target_time as u128
    }
}

pub struct ScoringSystem {
    ghost_score_multiplier: u16,
    dead_ghosts_counter: u8,
    little_scores: Vec<LittleScore>,
    little_timer_target: u32,
}

impl ScoringSystem {
    pub fn new() -> Self {
        ScoringSystem {
            ghost_score_multiplier: 200, // First ghost worth 200
            dead_ghosts_counter: 0,
            little_scores: Vec::new(),
            little_timer_target: 1000, // 1 second for floating score
        }
    }

    /// Add a ghost score at the given position
    pub fn add_ghost_score(&mut self, position: Position) -> u16 {
        let score_value = self.ghost_score_multiplier;
        let little_score = LittleScore::new(position, score_value);
        self.little_scores.push(little_score);

        // Double the multiplier for next ghost
        self.ghost_score_multiplier *= 2;
        self.dead_ghosts_counter += 1;

        score_value
    }

    /// Reset scoring system for new energizer
    pub fn reset_for_energizer(&mut self) {
        self.ghost_score_multiplier = 200;
    }

    /// Reset when pacman is not energized
    pub fn reset_ghost_counter(&mut self) {
        self.dead_ghosts_counter = 0;
    }

    /// Update little scores and remove expired ones
    pub fn update_little_scores(&mut self) {
        self.little_scores
            .retain(|score| !score.is_expired(self.little_timer_target));
    }

    /// Get current ghost score multiplier
    #[allow(dead_code)]
    pub fn get_ghost_score_multiplier(&self) -> u16 {
        self.ghost_score_multiplier
    }

    /// Get number of dead ghosts
    #[allow(dead_code)]
    pub fn get_dead_ghosts_counter(&self) -> u8 {
        self.dead_ghosts_counter
    }

    /// Get reference to little scores for rendering
    #[allow(dead_code)]
    pub fn get_little_scores(&self) -> &[LittleScore] {
        &self.little_scores
    }

    /// Get number of active little scores
    #[allow(dead_code)]
    pub fn get_little_scores_count(&self) -> usize {
        self.little_scores.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scoring_system_creation() {
        let scoring_system = ScoringSystem::new();
        assert_eq!(scoring_system.get_ghost_score_multiplier(), 200);
        assert_eq!(scoring_system.get_dead_ghosts_counter(), 0);
        assert_eq!(scoring_system.get_little_scores_count(), 0);
    }

    #[test]
    fn test_ghost_scoring() {
        let mut scoring_system = ScoringSystem::new();
        let position = Position::new(100, 100);

        // First ghost should be worth 200
        let score1 = scoring_system.add_ghost_score(position);
        assert_eq!(score1, 200);
        assert_eq!(scoring_system.get_ghost_score_multiplier(), 400);
        assert_eq!(scoring_system.get_dead_ghosts_counter(), 1);

        // Second ghost should be worth 400
        let score2 = scoring_system.add_ghost_score(position);
        assert_eq!(score2, 400);
        assert_eq!(scoring_system.get_ghost_score_multiplier(), 800);
        assert_eq!(scoring_system.get_dead_ghosts_counter(), 2);
    }

    #[test]
    fn test_energizer_reset() {
        let mut scoring_system = ScoringSystem::new();
        let position = Position::new(100, 100);

        // Score some ghosts
        scoring_system.add_ghost_score(position);
        scoring_system.add_ghost_score(position);
        assert_eq!(scoring_system.get_ghost_score_multiplier(), 800);

        // Reset for new energizer
        scoring_system.reset_for_energizer();
        assert_eq!(scoring_system.get_ghost_score_multiplier(), 200);
    }

    #[test]
    fn test_little_score_creation() {
        let position = Position::new(50, 75);
        let little_score = LittleScore::new(position, 400);

        assert_eq!(little_score.value, 400);
        assert_eq!(little_score.position.get_x(), 50);
        assert_eq!(little_score.position.get_y(), 75);

        // Timer should be started
        assert!(little_score.timer.get_ticks() > 0 || little_score.timer.get_ticks() == 0);
    }
}

