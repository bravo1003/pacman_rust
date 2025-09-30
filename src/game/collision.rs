use crate::board::BlockType;
use crate::entity::pacman::Pacman;
use crate::entity::{Blinky, Clyde, Entity, GhostBehavior, Inky, Pinky};
use crate::position::Position;

#[derive(Debug)]
pub enum CollisionEvent {
    PacmanEatsGhost {
        ghost_type: GhostType,
        position: Position,
    },
    GhostKillsPacman {
        ghost_type: GhostType,
    },
    NoCollision,
}

#[derive(Debug)]
pub enum GhostType {
    Blinky,
    Inky,
    Pinky,
    Clyde,
}

#[derive(Debug)]
pub enum FoodCollisionEvent {
    Nothing,
    Pellet,
    Energizer,
}

pub struct CollisionSystem;

impl CollisionSystem {
    pub fn new() -> Self {
        CollisionSystem
    }

    /// Check collision between Pacman and a specific ghost
    pub fn check_pacman_ghost_collision<'a>(
        &self,
        pacman: &Pacman<'a>,
        ghost: &impl GhostBehavior<'a>,
        ghost_type: GhostType,
        pacman_is_energized: bool,
    ) -> CollisionEvent {
        let pacman_pos = pacman.get_position();
        let ghost_pos = ghost.get_ghost().entity.get_position();

        if pacman.is_colliding(ghost_pos) && ghost.get_ghost().entity.is_alive() {
            if pacman_is_energized {
                CollisionEvent::PacmanEatsGhost {
                    ghost_type,
                    position: pacman_pos,
                }
            } else {
                CollisionEvent::GhostKillsPacman { ghost_type }
            }
        } else {
            CollisionEvent::NoCollision
        }
    }

    /// Check all ghost collisions and return the first collision found
    pub fn check_all_ghost_collisions<'a>(
        &self,
        pacman: &Pacman<'a>,
        blinky: &Blinky<'a>,
        inky: &Inky<'a>,
        pinky: &Pinky<'a>,
        clyde: &Clyde<'a>,
        pacman_is_energized: bool,
    ) -> Vec<CollisionEvent> {
        let mut collisions = Vec::new();

        // Check each ghost individually
        let blinky_collision = self.check_pacman_ghost_collision(
            pacman,
            blinky,
            GhostType::Blinky,
            pacman_is_energized,
        );
        if !matches!(blinky_collision, CollisionEvent::NoCollision) {
            collisions.push(blinky_collision);
        }

        let inky_collision =
            self.check_pacman_ghost_collision(pacman, inky, GhostType::Inky, pacman_is_energized);
        if !matches!(inky_collision, CollisionEvent::NoCollision) {
            collisions.push(inky_collision);
        }

        let pinky_collision =
            self.check_pacman_ghost_collision(pacman, pinky, GhostType::Pinky, pacman_is_energized);
        if !matches!(pinky_collision, CollisionEvent::NoCollision) {
            collisions.push(pinky_collision);
        }

        let clyde_collision =
            self.check_pacman_ghost_collision(pacman, clyde, GhostType::Clyde, pacman_is_energized);
        if !matches!(clyde_collision, CollisionEvent::NoCollision) {
            collisions.push(clyde_collision);
        }

        collisions
    }

    /// Check food collision and return the type of food consumed
    pub fn check_food_collision<'a>(
        &self,
        pacman: &Pacman<'a>,
        actual_map: &mut [BlockType],
    ) -> FoodCollisionEvent {
        match pacman.food_collision(actual_map) {
            0 => FoodCollisionEvent::Nothing,
            1 => FoodCollisionEvent::Energizer,
            _ => FoodCollisionEvent::Pellet,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collision_system_creation() {
        let _collision_system = CollisionSystem::new();
        // Test that we can create a collision system
        // No assertions needed - creation success is the test
    }

    #[test]
    fn test_collision_events() {
        // Test collision event variants
        let ghost_kill_event = CollisionEvent::GhostKillsPacman {
            ghost_type: GhostType::Blinky,
        };

        match ghost_kill_event {
            CollisionEvent::GhostKillsPacman { ghost_type } => {
                match ghost_type {
                    GhostType::Blinky => {} // Expected case
                    _ => panic!("Expected Blinky ghost type"),
                }
            }
            _ => panic!("Expected GhostKillsPacman event"),
        }
    }

    #[test]
    fn test_food_collision_events() {
        // Test that food collision events can be created and matched
        let events = [
            FoodCollisionEvent::Nothing,
            FoodCollisionEvent::Pellet,
            FoodCollisionEvent::Energizer,
        ];

        for event in events {
            match event {
                FoodCollisionEvent::Nothing => {}   // Valid variant
                FoodCollisionEvent::Pellet => {}    // Valid variant
                FoodCollisionEvent::Energizer => {} // Valid variant
            }
        }
    }

    #[test]
    fn test_ghost_types() {
        // Test that all ghost types can be created
        let ghost_types = [
            GhostType::Blinky,
            GhostType::Inky,
            GhostType::Pinky,
            GhostType::Clyde,
        ];

        assert_eq!(ghost_types.len(), 4);

        for ghost_type in ghost_types {
            // Test debug formatting works
            let debug_str = format!("{:?}", ghost_type);
            assert!(!debug_str.is_empty());
        }
    }
}

