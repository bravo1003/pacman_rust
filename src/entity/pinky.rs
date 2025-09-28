use crate::board::EntityType;
use crate::entity::{Entity, Ghost, GhostBehavior, GhostType};
use crate::position::Position;
use crate::BLOCK_SIZE_24;
use sdl2::pixels::Color;

pub struct Pinky<'a> {
    ghost: Ghost<'a>,
}

impl<'a> Pinky<'a> {
    pub fn new(
        texture_creator: &'a sdl2::render::TextureCreator<sdl2::video::WindowContext>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let scatter_target = Position::new((2 * BLOCK_SIZE_24) as i16, 0);
        let home_position = Position::new(
            (13 * BLOCK_SIZE_24 + BLOCK_SIZE_24 / 2) as i16,
            (17 * BLOCK_SIZE_24 + BLOCK_SIZE_24 / 2) as i16,
        );
        let color = Color::RGB(255, 192, 203); // Pink
        let ghost = Ghost::new(color, EntityType::Pinky, scatter_target, home_position, texture_creator)?;

        Ok(Pinky { ghost })
    }

    pub fn get_ghost_mut(&mut self) -> &mut Ghost<'a> {
        &mut self.ghost
    }

    pub fn get_ghost(&self) -> &Ghost<'a> {
        &self.ghost
    }
}

impl<'a> GhostBehavior<'a> for Pinky<'a> {
    fn get_ghost_type(&self) -> GhostType {
        GhostType::Pinky
    }

    fn get_scatter_target(&self) -> Position {
        self.ghost.scatter_target
    }

    fn calculate_target(&mut self, pacman_pos: Position, _blinky_pos: Option<Position>) {
        // Pinky: Ambush - target 4 tiles ahead of Pacman's direction
        // For now, just target Pacman directly (we can improve this later with direction info)
        self.ghost.target = pacman_pos;
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
