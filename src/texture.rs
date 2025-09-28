use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Texture, TextureCreator, WindowCanvas};
use sdl2::surface::Surface;
use sdl2::ttf::{Font, Sdl2TtfContext};
use sdl2::video::WindowContext;
use std::path::Path;

pub struct LTexture<'a> {
    texture: Option<Texture<'a>>,
    width: u32,
    height: u32,
}

impl<'a> LTexture<'a> {
    pub fn new() -> Self {
        LTexture {
            texture: None,
            width: 0,
            height: 0,
        }
    }

    pub fn load_from_file(
        &mut self,
        texture_creator: &'a TextureCreator<WindowContext>,
        path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Free existing texture
        self.free();

        // Load image at specified path using SDL2_image
        let mut surface: Surface = sdl2::image::LoadSurface::from_file(Path::new(path))
            .map_err(|e| format!("Unable to load image {}: {}", path, e))?;

        // Set color key (cyan background becomes transparent)
        surface.set_color_key(true, Color::RGB(0, 255, 255))?;

        // Create texture from surface
        let texture = texture_creator.create_texture_from_surface(&surface)?;

        // Get dimensions
        self.width = surface.width();
        self.height = surface.height();
        self.texture = Some(texture);

        Ok(())
    }

    pub fn load_from_rendered_text(
        &mut self,
        texture_creator: &'a TextureCreator<WindowContext>,
        text: &str,
        font: &Font,
        color: Color,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Free existing texture
        self.free();

        // Render text surface
        let text_surface = font
            .render(text)
            .solid(color)
            .map_err(|e| format!("Unable to render text surface: {}", e))?;

        // Create texture from surface
        let texture = texture_creator.create_texture_from_surface(&text_surface)?;

        // Get dimensions
        self.width = text_surface.width();
        self.height = text_surface.height();
        self.texture = Some(texture);

        Ok(())
    }

    pub fn free(&mut self) {
        self.texture = None;
        self.width = 0;
        self.height = 0;
    }

    pub fn set_color(
        &mut self,
        red: u8,
        green: u8,
        blue: u8,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref mut texture) = self.texture {
            texture.set_color_mod(red, green, blue);
        }
        Ok(())
    }

    pub fn set_alpha(&mut self, alpha: u8) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref mut texture) = self.texture {
            texture.set_alpha_mod(alpha);
        }
        Ok(())
    }

    pub fn render(
        &self,
        canvas: &mut WindowCanvas,
        x: i32,
        y: i32,
        clip: Option<Rect>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.render_with_facing(canvas, x, y, 0, clip)
    }
    
    pub fn render_with_facing(
        &self,
        canvas: &mut WindowCanvas,
        x: i32,
        y: i32,
        facing: u8,
        clip: Option<Rect>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref texture) = self.texture {
            let mut render_quad = Rect::new(x, y, self.width, self.height);

            // Set clip rendering dimensions
            if let Some(clip_rect) = clip {
                render_quad.set_width(clip_rect.width());
                render_quad.set_height(clip_rect.height());
            }

            // Convert facing to SDL angle (like C++ version)
            let angle = match facing {
                0 => 0.0,   // Right
                1 => 90.0,  // Down  
                2 => 180.0, // Left
                3 => 270.0, // Up
                _ => 0.0,
            };

            canvas.copy_ex(
                texture,
                clip,
                Some(render_quad),
                angle,
                None,  // Center point (use default center)
                false, // No horizontal flip
                false, // No vertical flip
            )?;
        }
        Ok(())
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }
}

