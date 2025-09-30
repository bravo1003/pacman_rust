use crate::board::{BlockType, Direction, EntityType};
use crate::position::Position;
use crate::{BLOCK_SIZE_24, BOARD_WIDTH, WINDOW_WIDTH};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Facing {
    Right = 0,
    Up = 1,
    Left = 2,
    Down = 3,
    Scared = 4,
}

impl Facing {
    pub fn from_direction(direction: Direction) -> Self {
        match direction {
            Direction::Right => Facing::Right,
            Direction::Up => Facing::Up,
            Direction::Left => Facing::Left,
            Direction::Down => Facing::Down,
            Direction::Nowhere => Facing::Right, // Default to right
        }
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

pub trait Entity {
    fn new(identity: EntityType) -> Self;
    #[allow(dead_code)]
    fn get_identity(&self) -> EntityType;
    fn get_speed(&self) -> u8;
    fn get_direction(&self) -> Direction;
    fn get_facing(&self) -> Facing;
    fn is_alive(&self) -> bool;

    fn mod_speed(&mut self, new_speed: u8);
    fn mod_direction(&mut self, new_direction: Direction);
    fn mod_life_statement(&mut self, new_life_statement: bool);

    fn get_position(&self) -> Position;
    fn set_position(&mut self, position: Position);
    fn get_x(&self) -> i16;
    fn get_y(&self) -> i16;
    fn mod_x(&mut self, new_x: i16);
    fn mod_y(&mut self, new_y: i16);

    fn get_possible_position(&self, mover: Direction) -> (i16, i16);
    fn char_board_pos(&self, side_dir: u8, cell_x: f32, cell_y: f32) -> Position;
    fn wall_collision(&self, x: i16, y: i16, actual_map: &[BlockType], can_use_door: bool) -> bool;
    fn move_entity(&mut self, mover: Direction);
    fn check_wrap(&mut self);
    fn is_colliding(&self, other: Position) -> bool;

    fn set_facing(&mut self, direction: Direction);
}

pub struct BaseEntity {
    pub position: Position,
    #[allow(dead_code)]
    pub identity: EntityType,
    pub speed: u8,
    pub direction: Direction,
    pub facing: Facing,
    pub life_statement: bool,
}

impl Entity for BaseEntity {
    fn new(identity: EntityType) -> Self {
        BaseEntity {
            position: Position::new(0, 0),
            identity,
            speed: 2,
            direction: Direction::Right,
            facing: Facing::Right,
            life_statement: true,
        }
    }

    fn get_identity(&self) -> EntityType {
        self.identity
    }

    fn get_speed(&self) -> u8 {
        self.speed
    }

    fn get_direction(&self) -> Direction {
        self.direction
    }

    fn get_facing(&self) -> Facing {
        self.facing
    }

    fn is_alive(&self) -> bool {
        self.life_statement
    }

    fn mod_speed(&mut self, new_speed: u8) {
        self.speed = new_speed;
    }

    fn mod_direction(&mut self, new_direction: Direction) {
        self.direction = new_direction;
    }

    fn mod_life_statement(&mut self, new_life_statement: bool) {
        self.life_statement = new_life_statement;
    }

    fn get_position(&self) -> Position {
        self.position
    }

    fn set_position(&mut self, position: Position) {
        self.position = position;
    }

    fn get_x(&self) -> i16 {
        self.position.get_x()
    }

    fn get_y(&self) -> i16 {
        self.position.get_y()
    }

    fn mod_x(&mut self, new_x: i16) {
        self.position.mod_x(new_x);
    }

    fn mod_y(&mut self, new_y: i16) {
        self.position.mod_y(new_y);
    }

    fn get_possible_position(&self, mover: Direction) -> (i16, i16) {
        let mut x = self.get_x();
        let mut y = self.get_y();

        match mover {
            Direction::Right => x += 1,
            Direction::Up => y -= 1,
            Direction::Left => x -= 1,
            Direction::Down => y += 1,
            Direction::Nowhere => {}
        }

        (x, y)
    }

    fn char_board_pos(&self, side_dir: u8, cell_x: f32, cell_y: f32) -> Position {
        match side_dir {
            0 => Position::new(cell_x.floor() as i16, cell_y.floor() as i16),
            1 => Position::new(cell_x.ceil() as i16, cell_y.floor() as i16),
            2 => Position::new(cell_x.floor() as i16, cell_y.ceil() as i16),
            3 => Position::new(cell_x.ceil() as i16, cell_y.ceil() as i16),
            _ => Position::new(cell_x.floor() as i16, cell_y.floor() as i16),
        }
    }

    fn wall_collision(&self, x: i16, y: i16, actual_map: &[BlockType], can_use_door: bool) -> bool {
        let cell_x = x as f32 / BLOCK_SIZE_24 as f32;
        let cell_y = y as f32 / BLOCK_SIZE_24 as f32;

        for side_dir in 0..4 {
            let board_pos = self.char_board_pos(side_dir, cell_x, cell_y);
            let board_x = (board_pos.get_x().abs() % BOARD_WIDTH as i16) as usize;
            let board_y = board_pos.get_y() as usize;

            if board_y < crate::BOARD_HEIGHT && board_x < BOARD_WIDTH {
                let index = BOARD_WIDTH * board_y + board_x;
                if index < actual_map.len() {
                    match actual_map[index] {
                        BlockType::Wall => return true,
                        BlockType::Door => {
                            if !can_use_door {
                                return true;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        false
    }

    fn move_entity(&mut self, mover: Direction) {
        match mover {
            Direction::Right => self.mod_x(self.get_x() + 1),
            Direction::Up => self.mod_y(self.get_y() - 1),
            Direction::Left => self.mod_x(self.get_x() - 1),
            Direction::Down => self.mod_y(self.get_y() + 1),
            Direction::Nowhere => {}
        }
    }

    fn check_wrap(&mut self) {
        if self.get_x() > (WINDOW_WIDTH + BLOCK_SIZE_24) as i16 {
            self.mod_x(-(BLOCK_SIZE_24 as i16));
        }
        if self.get_x() < -(BLOCK_SIZE_24 as i16) {
            self.mod_x((WINDOW_WIDTH + BLOCK_SIZE_24) as i16);
        }
    }

    fn is_colliding(&self, other: Position) -> bool {
        let block_size = BLOCK_SIZE_24 as i16;
        if other.get_x() > self.get_x() - block_size
            && other.get_x() < self.get_x() + block_size
            && other.get_y() > self.get_y() - block_size
            && other.get_y() < self.get_y() + block_size
        {
            return true;
        }
        false
    }

    fn set_facing(&mut self, direction: Direction) {
        self.facing = Facing::from_direction(direction);
    }
}
