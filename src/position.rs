#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub x: i16,
    pub y: i16,
}

impl Position {
    pub fn new(x: i16, y: i16) -> Self {
        Position { x, y }
    }

    pub fn get_x(&self) -> i16 {
        self.x
    }

    pub fn get_y(&self) -> i16 {
        self.y
    }

    #[allow(dead_code)]
    pub fn get_pos(&self) -> Position {
        *self
    }

    pub fn mod_x(&mut self, new_x: i16) {
        self.x = new_x;
    }

    pub fn mod_y(&mut self, new_y: i16) {
        self.y = new_y;
    }

    #[allow(dead_code)]
    pub fn mod_coords(&mut self, new_x: i16, new_y: i16) {
        self.x = new_x;
        self.y = new_y;
    }

    #[allow(dead_code)]
    pub fn mod_pos(&mut self, new_pos: Position) {
        self.x = new_pos.x;
        self.y = new_pos.y;
    }
}
