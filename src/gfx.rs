use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::io::Cursor;

use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{Texture, TextureCreator, WindowCanvas};
use sdl2::surface::Surface;
use sdl2::video::WindowContext;

use crate::assets::{fonts, textures};
use crate::geometry::{Point, SPRITE};
use crate::text::Font;

pub const WHITE: Color = Color::RGB(255, 255, 255);
pub const BLUE: Color = Color::RGB(0, 0, 255);
pub const RED: Color = Color::RGB(255, 0, 0);
const YELLOW: Color = Color::RGB(255, 255, 0);

/// Textures drawn whole.
#[derive(Clone, Copy)]
pub enum Image {
    Map,
    Pellet,
    Energizer,
    Door,
    Life,
    ScoreLabel,
    HighScoreLabel,
    Ready,
    GameOver,
}

impl Image {
    /// Every image, in declaration order.
    const ALL: [Self; 9] = [
        Self::Map,
        Self::Pellet,
        Self::Energizer,
        Self::Door,
        Self::Life,
        Self::ScoreLabel,
        Self::HighScoreLabel,
        Self::Ready,
        Self::GameOver,
    ];
}

/// Horizontal strips of 32x32 animation frames.
#[derive(Clone, Copy)]
pub enum Sprite {
    PacMan,
    PacManDeath,
    GhostBody,
    GhostEyes,
    Fruit,
}

impl Sprite {
    /// Every sprite, in declaration order.
    const ALL: [Self; 5] = [
        Self::PacMan,
        Self::PacManDeath,
        Self::GhostBody,
        Self::GhostEyes,
        Self::Fruit,
    ];

    fn png(self) -> &'static [u8] {
        match self {
            Self::PacMan => textures::PACMAN,
            Self::PacManDeath => textures::PACMAN_DEATH,
            Self::GhostBody => textures::GHOST_BODY,
            Self::GhostEyes => textures::GHOST_EYES,
            Self::Fruit => textures::FRUIT,
        }
    }
}

/// Numbers drawn in the large font, each re-rendered only when it changes.
#[derive(Clone, Copy)]
pub enum Counter {
    Score,
    HighScore,
}

/// The canvas plus every texture the game draws.
pub struct Gfx<'a> {
    canvas: WindowCanvas,
    creator: &'a TextureCreator<WindowContext>,
    images: Vec<Texture<'a>>,
    sprites: Vec<Texture<'a>>,
    large_font: Font,
    small_font: Font,
    counters: [Option<(u32, Texture<'a>)>; 2],
    small_numbers: HashMap<u32, Texture<'a>>,
}

impl<'a> Gfx<'a> {
    /// `creator` must come from `canvas`.
    pub fn new(
        canvas: WindowCanvas,
        creator: &'a TextureCreator<WindowContext>,
    ) -> Result<Self, String> {
        let large_font = Font::new(fonts::EMULOGIC, 24)?;
        let small_font = Font::new(fonts::VP_PIXEL, 20)?;
        let images = Image::ALL
            .into_iter()
            .map(|image| match image {
                Image::Map => png_texture(creator, textures::MAP),
                Image::Pellet => png_texture(creator, textures::PELLET),
                Image::Energizer => png_texture(creator, textures::ENERGIZER),
                Image::Door => png_texture(creator, textures::DOOR),
                Image::Life => png_texture(creator, textures::LIFE),
                Image::ScoreLabel => text_texture(creator, &large_font, "Score", WHITE),
                Image::HighScoreLabel => text_texture(creator, &large_font, "High Score", WHITE),
                Image::Ready => text_texture(creator, &large_font, "ready!", YELLOW),
                Image::GameOver => text_texture(creator, &large_font, "game  over", RED),
            })
            .collect::<Result<_, _>>()?;
        let sprites = Sprite::ALL
            .into_iter()
            .map(|sprite| png_texture(creator, sprite.png()))
            .collect::<Result<_, _>>()?;
        Ok(Self {
            canvas,
            creator,
            images,
            sprites,
            large_font,
            small_font,
            counters: [None, None],
            small_numbers: HashMap::new(),
        })
    }

    pub fn canvas(&self) -> &WindowCanvas {
        &self.canvas
    }

    pub fn clear(&mut self) {
        self.canvas.clear();
    }

    pub fn present(&mut self) {
        self.canvas.present();
    }

    pub fn image(&mut self, image: Image, at: Point) -> Result<(), String> {
        self.tinted_image(image, at, WHITE)
    }

    pub fn tinted_image(&mut self, image: Image, at: Point, tint: Color) -> Result<(), String> {
        let texture = &mut self.images[image as usize];
        texture.set_color_mod(tint.r, tint.g, tint.b);
        draw_texture(&mut self.canvas, texture, None, at, 0.0)
    }

    /// Draws animation `frame` of a sprite, rotated clockwise by `angle` degrees.
    pub fn sprite(
        &mut self,
        sprite: Sprite,
        frame: u32,
        at: Point,
        angle: f64,
    ) -> Result<(), String> {
        self.draw_sprite(sprite, frame, at, angle, WHITE)
    }

    pub fn tinted_sprite(
        &mut self,
        sprite: Sprite,
        frame: u32,
        at: Point,
        tint: Color,
    ) -> Result<(), String> {
        self.draw_sprite(sprite, frame, at, 0.0, tint)
    }

    fn draw_sprite(
        &mut self,
        sprite: Sprite,
        frame: u32,
        at: Point,
        angle: f64,
        tint: Color,
    ) -> Result<(), String> {
        let texture = &mut self.sprites[sprite as usize];
        texture.set_color_mod(tint.r, tint.g, tint.b);
        let size = SPRITE as u32;
        let clip = Rect::new(frame as i32 * SPRITE, 0, size, size);
        draw_texture(&mut self.canvas, texture, Some(clip), at, angle)
    }

    /// Draws a number in the large font.
    pub fn counter(&mut self, counter: Counter, value: u32, at: Point) -> Result<(), String> {
        let slot = &mut self.counters[counter as usize];
        if slot.as_ref().is_none_or(|(shown, _)| *shown != value) {
            let texture = text_texture(self.creator, &self.large_font, &value.to_string(), WHITE)?;
            *slot = Some((value, texture));
        }
        let (_, texture) = slot.as_ref().expect("counter texture was just rendered");
        draw_texture(&mut self.canvas, texture, None, at, 0.0)
    }

    /// Draws a number in the small font, used for points popups.
    pub fn small_number(&mut self, value: u32, at: Point) -> Result<(), String> {
        let texture = match self.small_numbers.entry(value) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(text_texture(
                self.creator,
                &self.small_font,
                &value.to_string(),
                WHITE,
            )?),
        };
        draw_texture(&mut self.canvas, texture, None, at, 0.0)
    }
}

/// Copies a texture (or one clip of it) at its natural size.
fn draw_texture(
    canvas: &mut WindowCanvas,
    texture: &Texture,
    clip: Option<Rect>,
    at: Point,
    angle: f64,
) -> Result<(), String> {
    let (width, height) = match clip {
        Some(clip) => (clip.width(), clip.height()),
        None => {
            let query = texture.query();
            (query.width, query.height)
        }
    };
    let dst = Rect::new(at.x, at.y, width, height);
    canvas.copy_ex(texture, clip, dst, angle, None, false, false)
}

fn png_texture<'a>(
    creator: &'a TextureCreator<WindowContext>,
    data: &[u8],
) -> Result<Texture<'a>, String> {
    let mut decoder = png::Decoder::new(Cursor::new(data));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::ALPHA);
    let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
    let mut pixels = vec![0; reader.output_buffer_size().ok_or("PNG is too large")?];
    let info = reader.next_frame(&mut pixels).map_err(|e| e.to_string())?;
    if (info.color_type, info.bit_depth) != (png::ColorType::Rgba, png::BitDepth::Eight) {
        return Err(format!(
            "unsupported PNG layout {:?} {:?}",
            info.color_type, info.bit_depth
        ));
    }
    rgba_texture(creator, &mut pixels, info.width, info.height)
}

/// Renders text the way `TTF_RenderText_Solid` does: opaque `color` glyphs
/// on a transparent background.
fn text_texture<'a>(
    creator: &'a TextureCreator<WindowContext>,
    font: &Font,
    text: &str,
    color: Color,
) -> Result<Texture<'a>, String> {
    let bitmap = font.render(text);
    let mut pixels: Vec<u8> = bitmap
        .pixels
        .iter()
        .flat_map(|&set| {
            if set {
                [color.r, color.g, color.b, 255]
            } else {
                [0; 4]
            }
        })
        .collect();
    rgba_texture(creator, &mut pixels, bitmap.width, bitmap.height)
}

fn rgba_texture<'a>(
    creator: &'a TextureCreator<WindowContext>,
    pixels: &mut [u8],
    width: u32,
    height: u32,
) -> Result<Texture<'a>, String> {
    let surface = Surface::from_data(pixels, width, height, width * 4, PixelFormatEnum::RGBA32)?;
    creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())
}
