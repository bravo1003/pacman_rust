use crate::texture::GameTexture;
use crate::{
    BLOCK_SIZE_24, BLOCK_SIZE_32, BLUE, BOARD_HEIGHT, BOARD_WIDTH, WHITE, WINDOW_HEIGHT,
    WINDOW_WIDTH,
};
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::ttf::{Font, Sdl2TtfContext};
use sdl2::video::WindowContext;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlockType {
    Wall,
    Door,
    Pellet,
    Energizer,
    Nothing,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Right,
    Up,
    Left,
    Down,
    #[allow(dead_code)]
    Nowhere,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EntityType {
    PacMan,
    Blinky,
    Inky,
    Pinky,
    Clyde,
    #[allow(dead_code)]
    None,
}

pub struct Board<'a> {
    numeric_board: [BlockType; BOARD_HEIGHT * BOARD_WIDTH],
    score: u32,
    lives: i8,
    #[allow(dead_code)]
    is_extra: bool,

    map_texture: GameTexture<'a>,
    pellet_texture: GameTexture<'a>,
    energizer_texture: GameTexture<'a>,
    door_texture: GameTexture<'a>,
    lives_texture: GameTexture<'a>,

    score_word_texture: GameTexture<'a>,
    score_texture: GameTexture<'a>,
    high_score_word_texture: GameTexture<'a>,
    high_score_texture: GameTexture<'a>,
}

impl<'a> Board<'a> {
    pub const CHAR_BOARD: &'static str = concat!(
        "                            ",
        "                            ",
        "                            ",
        "############################",
        "#............##............#",
        "#.####.#####.##.#####.####.#",
        "#o####.#####.##.#####.####o#",
        "#.####.#####.##.#####.####.#",
        "#..........................#",
        "#.####.##.########.##.####.#",
        "#.####.##.########.##.####.#",
        "#......##....##....##......#",
        "######.##### ## #####.######",
        "     #.##### ## #####.#     ",
        "     #.##    1     ##.#     ",
        "     #.## ###==### ##.#     ",
        "######.## #      # ##.######",
        "      .   #2 3 4 #   .      ",
        "######.## #      # ##.######",
        "     #.## ######## ##.#     ",
        "     #.##          ##.#     ",
        "     #.## ######## ##.#     ",
        "######.## ######## ##.######",
        "#............##............#",
        "#.####.#####.##.#####.####.#",
        "#.####.#####.##.#####.####.#",
        "#o..##.......0 .......##..o#",
        "###.##.##.########.##.##.###",
        "###.##.##.########.##.##.###",
        "#......##....##....##......#",
        "#.##########.##.##########.#",
        "#.##########.##.##########.#",
        "#..........................#",
        "############################",
        "                            ",
        "                            "
    );
    pub fn new(
        texture_creator: &'a TextureCreator<WindowContext>,
        ttf_context: &'a Sdl2TtfContext,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let font = ttf_context.load_font("assets/emulogic.ttf", 24)?;

        let mut board = Board {
            numeric_board: [BlockType::Nothing; BOARD_HEIGHT * BOARD_WIDTH],
            score: 0,
            lives: 4,
            is_extra: false,
            map_texture: GameTexture::new(),
            pellet_texture: GameTexture::new(),
            energizer_texture: GameTexture::new(),
            door_texture: GameTexture::new(),
            lives_texture: GameTexture::new(),
            score_word_texture: GameTexture::new(),
            score_texture: GameTexture::new(),
            high_score_word_texture: GameTexture::new(),
            high_score_texture: GameTexture::new(),
        };

        board
            .map_texture
            .load_from_file(texture_creator, "assets/Map24.png")?;
        board
            .pellet_texture
            .load_from_file(texture_creator, "assets/Pellet24.png")?;
        board
            .energizer_texture
            .load_from_file(texture_creator, "assets/Energizer24.png")?;
        board
            .door_texture
            .load_from_file(texture_creator, "assets/Door.png")?;
        board
            .lives_texture
            .load_from_file(texture_creator, "assets/Lives32.png")?;

        board
            .score_word_texture
            .load_from_rendered_text(texture_creator, "Score", &font, WHITE)?;
        board.high_score_word_texture.load_from_rendered_text(
            texture_creator,
            "High Score",
            &font,
            WHITE,
        )?;

        board.map_texture.set_color(BLUE.r, BLUE.g, BLUE.b)?;

        board.convert_sketch();
        board.set_score(texture_creator, &font)?;
        board.set_high_score(texture_creator, &font)?;

        Ok(board)
    }

    fn convert_sketch(&mut self) {
        let chars: Vec<char> = Self::CHAR_BOARD.chars().collect();
        for i in 0..BOARD_HEIGHT * BOARD_WIDTH {
            if i < chars.len() {
                self.numeric_board[i] = match chars[i] {
                    '#' => BlockType::Wall,
                    '=' => BlockType::Door,
                    '.' => BlockType::Pellet,
                    'o' => BlockType::Energizer,
                    _ => BlockType::Nothing,
                };
            }
        }
    }

    #[allow(dead_code)]
    pub fn get_block_type(&self, x: usize, y: usize) -> BlockType {
        if x >= BOARD_WIDTH || y >= BOARD_HEIGHT {
            return BlockType::Wall;
        }
        self.numeric_board[y * BOARD_WIDTH + x]
    }

    pub fn copy_board(&self, actual_map: &mut [BlockType]) {
        actual_map.copy_from_slice(&self.numeric_board);
    }

    pub fn reset_position(&self, entity_type: EntityType) -> crate::position::Position {
        let chars: Vec<char> = Self::CHAR_BOARD.chars().collect();

        let target_char = match entity_type {
            EntityType::PacMan => '0',
            EntityType::Blinky => '1',
            EntityType::Inky => '2',
            EntityType::Pinky => '3',
            EntityType::Clyde => '4',
            EntityType::None => return crate::position::Position::new(0, 0),
        };

        for (i, &ch) in chars.iter().enumerate() {
            if ch == target_char {
                let x = (i % BOARD_WIDTH) as u32 * BLOCK_SIZE_24 + BLOCK_SIZE_24 / 2;
                let y = (i / BOARD_WIDTH) as u32 * BLOCK_SIZE_24;
                return crate::position::Position::new(x as i16, y as i16);
            }
        }

        crate::position::Position::new(0, 0)
    }

    pub fn draw(
        &mut self,
        canvas: &mut WindowCanvas,
        actual_map: &[BlockType],
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.score_word_texture.render(canvas, 0, 0, None)?;
        self.score_texture
            .render(canvas, 0, BLOCK_SIZE_32 as i32, None)?;
        self.high_score_word_texture.render(canvas, 336, 0, None)?;
        self.high_score_texture
            .render(canvas, 336, BLOCK_SIZE_32 as i32, None)?;

        self.map_texture.render(canvas, 0, 0, None)?;

        let door_x = (WINDOW_WIDTH / 2) as i32 - 23;
        let door_y = (WINDOW_HEIGHT / 2) as i32 - 57;
        self.door_texture.render(canvas, door_x, door_y, None)?;

        for y in 0..BOARD_HEIGHT {
            for x in 0..BOARD_WIDTH {
                let index = y * BOARD_WIDTH + x;
                let block_type = actual_map[index];

                let render_x = (x as u32 * BLOCK_SIZE_24) as i32;
                let render_y = (y as u32 * BLOCK_SIZE_24) as i32;

                match block_type {
                    BlockType::Pellet => {
                        self.pellet_texture
                            .render(canvas, render_x, render_y, None)?;
                    }
                    BlockType::Energizer => {
                        self.energizer_texture
                            .render(canvas, render_x, render_y, None)?;
                    }
                    _ => {}
                }
            }
        }

        for i in 1..=self.lives {
            if i > 0 {
                let lives_x = (i as u32 * BLOCK_SIZE_32) as i32;
                let lives_y = (26 * BLOCK_SIZE_32 - BLOCK_SIZE_32 / 4) as i32;
                self.lives_texture.render(canvas, lives_x, lives_y, None)?;
            }
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_score(&self) -> u32 {
        self.score
    }

    pub fn get_lives(&self) -> i8 {
        self.lives
    }

    pub fn score_increase(&mut self, points: u16) {
        match points {
            0 => self.score += 10,
            1 => self.score += 50,
            _ => self.score += points as u32,
        }
    }

    pub fn set_score(
        &mut self,
        texture_creator: &'a TextureCreator<WindowContext>,
        font: &Font,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let score_text = format!("{}", self.score);
        self.score_texture
            .load_from_rendered_text(texture_creator, &score_text, font, WHITE)?;
        Ok(())
    }

    pub fn set_high_score(
        &mut self,
        texture_creator: &'a TextureCreator<WindowContext>,
        font: &Font,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let high_score = self.score.max(0);
        let high_score_text = format!("{}", high_score);
        self.high_score_texture.load_from_rendered_text(
            texture_creator,
            &high_score_text,
            font,
            WHITE,
        )?;
        Ok(())
    }

    pub fn decrease_lives(&mut self) {
        if self.lives > 0 {
            self.lives -= 1;
        }
    }

    pub fn score_increase_by_value(&mut self, value: u16) {
        self.score += value as u32;
    }
}
