use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::time::{Duration, Instant};

mod board;
mod entity;
mod game;
mod position;
mod texture;

use game::Game;

pub const BOARD_WIDTH: usize = 28;
pub const BOARD_HEIGHT: usize = 36;
pub const BLOCK_SIZE_24: u32 = 24;
pub const BLOCK_SIZE_32: u32 = 32;
pub const WINDOW_WIDTH: u32 = BOARD_WIDTH as u32 * BLOCK_SIZE_24;
pub const WINDOW_HEIGHT: u32 = BOARD_HEIGHT as u32 * BLOCK_SIZE_24;

pub const BLACK: Color = Color::RGB(0, 0, 0);
pub const WHITE: Color = Color::RGB(255, 255, 255);
pub const YELLOW: Color = Color::RGB(255, 255, 0);
pub const RED: Color = Color::RGB(255, 0, 0);
pub const CYAN: Color = Color::RGB(0, 192, 255);
pub const PINK: Color = Color::RGB(255, 192, 203);
pub const ORANGE: Color = Color::RGB(255, 128, 0);
pub const BLUE: Color = Color::RGB(0, 0, 255);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let _image_context = sdl2::image::init(sdl2::image::InitFlag::PNG)?;

    let ttf_context =
        sdl2::ttf::init().map_err(|e| format!("SDL2_TTF initialization failed: {}", e))?;

    let window = video_subsystem
        .window("Pacman", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()?;

    let mut canvas = window.into_canvas().present_vsync().build()?;
    let texture_creator = canvas.texture_creator();

    let font = ttf_context.load_font("assets/emulogic.ttf", 24)?;

    let mut game = Game::new(&texture_creator, &ttf_context)?;

    let mut event_pump = sdl_context.event_pump()?;
    let mut running = true;
    let target_fps = 60;
    let frame_duration = Duration::from_millis(1000 / target_fps);

    while running {
        let frame_start = Instant::now();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    running = false;
                }
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    game.handle_input(keycode);
                }
                _ => {}
            }
        }

        game.update();

        canvas.set_draw_color(BLACK);
        canvas.clear();

        game.draw(&mut canvas, &texture_creator, &font)?;

        canvas.present();

        let frame_time = frame_start.elapsed();
        if frame_time < frame_duration {
            std::thread::sleep(frame_duration - frame_time);
        }
    }

    Ok(())
}
