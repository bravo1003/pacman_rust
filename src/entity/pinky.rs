use crate::board::{Direction, EntityType};
use crate::entity::{Entity, Ghost, GhostBehavior, GhostType};
use crate::position::Position;
use crate::{BLOCK_SIZE_24, PINK};

pub struct Pinky<'a> {
    ghost: Ghost<'a>,
}

impl<'a> Pinky<'a> {
    pub fn new(
        texture_creator: &'a sdl2::render::TextureCreator<sdl2::video::WindowContext>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let scatter_target = Position::new(
            (2 * BLOCK_SIZE_24 + BLOCK_SIZE_24 / 2) as i16,
            (BLOCK_SIZE_24 / 2) as i16,
        );
        let home_position = Position::new(
            (13 * BLOCK_SIZE_24 + BLOCK_SIZE_24 / 2) as i16,
            (17 * BLOCK_SIZE_24 + BLOCK_SIZE_24 / 2) as i16,
        );
        let color = PINK;
        let mut ghost = Ghost::new(
            color,
            EntityType::Pinky,
            scatter_target,
            home_position,
            texture_creator,
        )?;

        ghost.entity.set_facing(Direction::Down);
        Ok(Pinky { ghost })
    }

    pub fn get_ghost_mut(&mut self) -> &mut Ghost<'a> {
        &mut self.ghost
    }

    #[allow(dead_code)]
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

    fn calculate_target(
        &mut self,
        pacman_pos: Position,
        pacman_dir: Direction,
        _blinky_pos: Option<Position>,
    ) {
        let offset = BLOCK_SIZE_24 * 4;

        let target_pos = match pacman_dir {
            Direction::Up => Position::new(pacman_pos.get_x(), pacman_pos.get_y() - offset as i16),
            Direction::Down => {
                Position::new(pacman_pos.get_x(), pacman_pos.get_y() + offset as i16)
            }
            Direction::Left => {
                Position::new(pacman_pos.get_x() - offset as i16, pacman_pos.get_y())
            }
            Direction::Right => {
                Position::new(pacman_pos.get_x() + offset as i16, pacman_pos.get_y())
            }
            Direction::Nowhere => pacman_pos,
        };

        self.ghost.target = target_pos;
    }

    #[allow(dead_code)]
    fn get_can_use_door(&self) -> bool {
        self.ghost.can_use_door
    }

    #[allow(dead_code)]
    fn set_can_use_door(&mut self, can_use_door: bool) {
        self.ghost.can_use_door = can_use_door;
    }

    fn get_ghost_mut(&mut self) -> &mut Ghost<'a> {
        &mut self.ghost
    }

    #[allow(dead_code)]
    fn get_ghost(&self) -> &Ghost<'a> {
        &self.ghost
    }
}
