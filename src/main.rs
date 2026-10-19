//! Use the arrow keys or WASD to move.

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use rust_pacman::{Audio, Direction, Game, Gfx, WINDOW_HEIGHT, WINDOW_WIDTH};

const HIGH_SCORE_FILE: &str = "HighScore.txt";
const FRAME_MS: f64 = 16.666;

fn main() -> Result<(), String> {
    let sdl = sdl2::init()?;
    let timer = sdl.timer()?;
    let window = sdl
        .video()?
        .window("PacMan", WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;
    let canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;
    let creator = canvas.texture_creator();
    let mut gfx = Gfx::new(canvas, &creator)?;
    let mut audio = Audio::new(&sdl);
    let mut events = sdl.event_pump()?;
    let mut game = Game::new(timer.ticks(), HIGH_SCORE_FILE);

    loop {
        let frame_start = timer.performance_counter();
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } => return Ok(()),
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    if let Some(direction) = direction_for(key) {
                        game.steer(direction);
                    }
                }
                _ => {}
            }
        }

        let now = timer.ticks();
        if game.update(now) {
            gfx.clear();
            game.draw(&mut gfx, now)?;
            gfx.present();
        }
        audio.apply(game.drain_sounds());

        // Run at about 60 frames per second, as everything moves per frame.
        let elapsed = timer.performance_counter() - frame_start;
        let elapsed_ms = elapsed as f64 * 1000.0 / timer.performance_frequency() as f64;
        let delay_ms = FRAME_MS - elapsed_ms;
        if delay_ms > 0.0 {
            timer.delay(delay_ms as u32);
        }
    }
}

fn direction_for(key: Keycode) -> Option<Direction> {
    match key {
        Keycode::RIGHT | Keycode::D => Some(Direction::Right),
        Keycode::UP | Keycode::W => Some(Direction::Up),
        Keycode::LEFT | Keycode::A => Some(Direction::Left),
        Keycode::DOWN | Keycode::S => Some(Direction::Down),
        _ => None,
    }
}
