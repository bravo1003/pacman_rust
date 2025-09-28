use crate::board::EntityType;
use crate::entity::{Entity, Ghost, GhostBehavior, GhostType};
use crate::position::Position;
use crate::BLOCK_SIZE_24;
use sdl2::pixels::Color;

pub struct Inky<'a> {
    ghost: Ghost<'a>,
}

impl<'a> Inky<'a> {
    pub fn new(
        texture_creator: &'a sdl2::render::TextureCreator<sdl2::video::WindowContext>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let scatter_target = Position::new(
            (26 * BLOCK_SIZE_24 + BLOCK_SIZE_24 / 2) as i16,
            (35 * BLOCK_SIZE_24 + BLOCK_SIZE_24 / 2) as i16
        );
        let home_position = Position::new(
            (11 * BLOCK_SIZE_24 + BLOCK_SIZE_24 / 2) as i16,
            (17 * BLOCK_SIZE_24 + BLOCK_SIZE_24 / 2) as i16,
        );
        let color = Color::RGB(0, 255, 255); // Cyan
        let ghost = Ghost::new(color, EntityType::Inky, scatter_target, home_position, texture_creator)?;

        Ok(Inky { ghost })
    }

    pub fn get_ghost_mut(&mut self) -> &mut Ghost<'a> {
        &mut self.ghost
    }

    pub fn get_ghost(&self) -> &Ghost<'a> {
        &self.ghost
    }
}

impl<'a> GhostBehavior<'a> for Inky<'a> {
    fn get_ghost_type(&self) -> GhostType {
        GhostType::Inky
    }

    fn get_scatter_target(&self) -> Position {
        self.ghost.scatter_target
    }

    fn calculate_target(&mut self, pacman_pos: Position, blinky_pos: Option<Position>) {
        // Inky: Complex AI using Blinky's position
        if let Some(blinky_position) = blinky_pos {
            // Vector from Blinky to Pacman, doubled
            let target_x = pacman_pos.get_x() + (pacman_pos.get_x() - blinky_position.get_x());
            let target_y = pacman_pos.get_y() + (pacman_pos.get_y() - blinky_position.get_y());
            self.ghost.target = Position::new(target_x, target_y);
        } else {
            // Fallback: direct chase if Blinky position not available
            self.ghost.target = pacman_pos;
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
