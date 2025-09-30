use crate::board::{Direction, EntityType};
use crate::entity::{Entity, Ghost, GhostBehavior, GhostType};
use crate::position::Position;
use crate::{BLOCK_SIZE_24, RED};

pub struct Blinky<'a> {
    ghost: Ghost<'a>,
}

impl<'a> Blinky<'a> {
    pub fn new(
        texture_creator: &'a sdl2::render::TextureCreator<sdl2::video::WindowContext>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let scatter_target = Position::new(
            (25 * BLOCK_SIZE_24 + BLOCK_SIZE_24 / 2) as i16,
            (BLOCK_SIZE_24 / 2) as i16,
        );
        let home_position = Position::new(
            (13 * BLOCK_SIZE_24 + BLOCK_SIZE_24 / 2) as i16,
            (17 * BLOCK_SIZE_24 + BLOCK_SIZE_24 / 2) as i16,
        );
        let color = RED;
        let mut ghost = Ghost::new(
            color,
            EntityType::Blinky,
            scatter_target,
            home_position,
            texture_creator,
        )?;

        ghost.entity.set_facing(Direction::Up);
        Ok(Blinky { ghost })
    }

    pub fn get_ghost_mut(&mut self) -> &mut Ghost<'a> {
        &mut self.ghost
    }

    pub fn get_ghost(&self) -> &Ghost<'a> {
        &self.ghost
    }
}

impl<'a> GhostBehavior<'a> for Blinky<'a> {
    fn get_ghost_type(&self) -> GhostType {
        GhostType::Blinky
    }

    fn get_scatter_target(&self) -> Position {
        self.ghost.scatter_target
    }

    fn calculate_target(
        &mut self,
        pacman_pos: Position,
        _pacman_dir: Direction,
        _blinky_pos: Option<Position>,
    ) {
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
