use crate::board::{Direction, EntityType};
use crate::entity::{Entity, Ghost, GhostBehavior, GhostType};
use crate::position::Position;
use crate::{BLOCK_SIZE_24, ORANGE, WINDOW_WIDTH};

pub struct Clyde<'a> {
    ghost: Ghost<'a>,
}

impl<'a> Clyde<'a> {
    pub fn new(
        texture_creator: &'a sdl2::render::TextureCreator<sdl2::video::WindowContext>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let scatter_target = Position::new(
            (BLOCK_SIZE_24 / 2) as i16,
            (35 * BLOCK_SIZE_24 + BLOCK_SIZE_24 / 2) as i16,
        );
        let home_position = Position::new(
            (15 * BLOCK_SIZE_24 + BLOCK_SIZE_24 / 2) as i16,
            (17 * BLOCK_SIZE_24 + BLOCK_SIZE_24 / 2) as i16,
        );
        let color = ORANGE;
        let mut ghost = Ghost::new(
            color,
            EntityType::Clyde,
            scatter_target,
            home_position,
            texture_creator,
        )?;

        ghost.entity.set_facing(Direction::Up);
        Ok(Clyde { ghost })
    }

    pub fn get_ghost_mut(&mut self) -> &mut Ghost<'a> {
        &mut self.ghost
    }

    #[allow(dead_code)]
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

    fn calculate_target(
        &mut self,
        pacman_pos: Position,
        _pacman_dir: Direction,
        _blinky_pos: Option<Position>,
    ) {
        let mut dist_x = (self.ghost.entity.get_x() - pacman_pos.get_x()).abs();
        if dist_x > (WINDOW_WIDTH / 2) as i16 {
            dist_x = WINDOW_WIDTH as i16 - dist_x;
        }
        let dist = ((dist_x as f32).powi(2)
            + ((self.ghost.entity.get_y() - pacman_pos.get_y()) as f32).powi(2))
        .sqrt();

        if dist > (8 * BLOCK_SIZE_24) as f32 {
            self.ghost.target = pacman_pos;
        } else {
            self.ghost.target = self.ghost.scatter_target;
        }
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
