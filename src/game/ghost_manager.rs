use crate::board::{BlockType, Direction, EntityType};
use crate::entity::pacman::Pacman;
use crate::entity::{Blinky, Clyde, Entity, GhostBehavior, Inky, Pinky};
use sdl2::render::{Canvas, TextureCreator};
use sdl2::video::WindowContext;

/// Simplified ghost management system for all ghosts
#[allow(dead_code)]
pub struct GhostManager<'a> {
    pub blinky: Blinky<'a>,
    pub inky: Inky<'a>,
    pub pinky: Pinky<'a>,
    pub clyde: Clyde<'a>,
}

#[allow(dead_code)]
impl<'a> GhostManager<'a> {
    /// Create new ghost manager with all ghosts
    pub fn new(
        texture_creator: &'a TextureCreator<WindowContext>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let blinky = Blinky::new(texture_creator)?;
        let inky = Inky::new(texture_creator)?;
        let pinky = Pinky::new(texture_creator)?;
        let clyde = Clyde::new(texture_creator)?;

        Ok(GhostManager {
            blinky,
            inky,
            pinky,
            clyde,
        })
    }

    /// Update all ghost positions
    pub fn update_all_ghosts(
        &mut self,
        actual_map: &[BlockType],
        pacman: &Pacman,
        is_scatter_mode: bool,
    ) {
        // Get blinky position for inky's special targeting
        let blinky_pos = self.blinky.get_ghost().entity.get_position();

        self.blinky
            .update_pos(actual_map, pacman, None, is_scatter_mode);
        self.inky
            .update_pos(actual_map, pacman, Some(blinky_pos), is_scatter_mode);
        self.pinky
            .update_pos(actual_map, pacman, None, is_scatter_mode);
        self.clyde
            .update_pos(actual_map, pacman, None, is_scatter_mode);
    }

    /// Draw all ghosts
    pub fn draw_all_ghosts(
        &mut self,
        canvas: &mut Canvas<sdl2::video::Window>,
        pacman_energized: bool,
        ghost_ticks: u128,
        ghost_timer_target: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.blinky
            .get_ghost_mut()
            .draw(canvas, pacman_energized, ghost_ticks, ghost_timer_target)?;
        self.inky
            .get_ghost_mut()
            .draw(canvas, pacman_energized, ghost_ticks, ghost_timer_target)?;
        self.pinky
            .get_ghost_mut()
            .draw(canvas, pacman_energized, ghost_ticks, ghost_timer_target)?;
        self.clyde
            .get_ghost_mut()
            .draw(canvas, pacman_energized, ghost_ticks, ghost_timer_target)?;
        Ok(())
    }

    /// Reset all ghost life statements (alive)
    pub fn reset_all_ghost_life_statements(&mut self) {
        self.blinky.get_ghost_mut().entity.mod_life_statement(true);
        self.inky.get_ghost_mut().entity.mod_life_statement(true);
        self.pinky.get_ghost_mut().entity.mod_life_statement(true);
        self.clyde.get_ghost_mut().entity.mod_life_statement(true);
    }

    /// Reset all ghost facing directions
    pub fn reset_all_ghost_facing(&mut self) {
        self.blinky.get_ghost_mut().entity.set_facing(Direction::Left);
        self.inky.get_ghost_mut().entity.set_facing(Direction::Up);
        self.pinky.get_ghost_mut().entity.set_facing(Direction::Down);
        self.clyde.get_ghost_mut().entity.set_facing(Direction::Up);
    }

    /// Set all ghost positions to their home positions
    pub fn reset_all_ghost_positions(&mut self, board: &crate::board::Board) {
        let blinky_start = board.reset_position(EntityType::Blinky);
        self.blinky.get_ghost_mut().entity.set_position(blinky_start);

        let inky_start = board.reset_position(EntityType::Inky);
        self.inky.get_ghost_mut().entity.set_position(inky_start);

        let pinky_start = board.reset_position(EntityType::Pinky);
        self.pinky.get_ghost_mut().entity.set_position(pinky_start);

        let clyde_start = board.reset_position(EntityType::Clyde);
        self.clyde.get_ghost_mut().entity.set_position(clyde_start);
    }

    /// Get blinky for individual access
    pub fn get_blinky_mut(&mut self) -> &mut Blinky<'a> {
        &mut self.blinky
    }

    /// Get inky for individual access
    pub fn get_inky_mut(&mut self) -> &mut Inky<'a> {
        &mut self.inky
    }

    /// Get pinky for individual access
    pub fn get_pinky_mut(&mut self) -> &mut Pinky<'a> {
        &mut self.pinky
    }

    /// Get clyde for individual access
    pub fn get_clyde_mut(&mut self) -> &mut Clyde<'a> {
        &mut self.clyde
    }
}
