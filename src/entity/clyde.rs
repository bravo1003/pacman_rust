use crate::board::EntityType;
use crate::entity::{Entity, Ghost, GhostBehavior, GhostType};
use crate::position::Position;
use crate::{BLOCK_SIZE_24, WINDOW_WIDTH};
use sdl2::pixels::Color;

pub struct Clyde<'a> {
    ghost: Ghost<'a>,
}

impl<'a> Clyde<'a> {
    pub fn new(
        texture_creator: &'a sdl2::render::TextureCreator<sdl2::video::WindowContext>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let scatter_target = Position::new(
            (BLOCK_SIZE_24 / 2) as i16,
            (35 * BLOCK_SIZE_24 + BLOCK_SIZE_24 / 2) as i16
        );
        let home_position = Position::new(
            (15 * BLOCK_SIZE_24 + BLOCK_SIZE_24 / 2) as i16,
            (17 * BLOCK_SIZE_24 + BLOCK_SIZE_24 / 2) as i16,
        );
        let color = Color::RGB(255, 165, 0); // Orange
        let ghost = Ghost::new(color, EntityType::Clyde, scatter_target, home_position, texture_creator)?;

        Ok(Clyde { ghost })
    }

    pub fn get_ghost_mut(&mut self) -> &mut Ghost<'a> {
        &mut self.ghost
    }

    pub fn get_ghost(&self) -> &Ghost<'a> {
        &self.ghost
    }
}

impl<'a> GhostBehavior<'a> for Clyde<'a> {
    fn get_ghost_type(&self) -> GhostType {
        GhostType::Clyde
    }

    fn get_scatter_target(&self) -> Position {
        self.ghost.scatter_target
    }

    fn calculate_target(&mut self, pacman_pos: Position, _blinky_pos: Option<Position>) {
        // Clyde: Shy - chase when far, scatter when close
        let mut dist_x = (self.ghost.entity.get_x() - pacman_pos.get_x()).abs();
        if dist_x > (WINDOW_WIDTH / 2) as i16 {
            dist_x = WINDOW_WIDTH as i16 - dist_x;
        }
        let dist = ((dist_x as f32).powi(2)
            + ((self.ghost.entity.get_y() - pacman_pos.get_y()) as f32).powi(2))
        .sqrt();

        if dist > (8 * BLOCK_SIZE_24) as f32 {
            // Far away: chase Pacman
            self.ghost.target = pacman_pos;
        } else {
            // Close: scatter to corner
            self.ghost.target = self.ghost.scatter_target;
        }
    }

    fn get_can_use_door(&self) -> bool {
        self.ghost.can_use_door
    }

    fn set_can_use_door(&mut self, can_use_door: bool) {
        self.ghost.can_use_door = can_use_door;
    }

    fn get_ghost_mut(&mut self) -> &mut Ghost<'a> {
        &mut self.ghost
    }

    fn get_ghost(&self) -> &Ghost<'a> {
        &self.ghost
    }
}
