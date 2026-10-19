//! Pac-Man, a faithful Rust port of the `PacMan_SDL` C++ project.
//!
//! Game logic is independent of SDL: [`Game`] advances one frame per
//! [`Game::update`] call using a millisecond clock supplied by the caller,
//! and reports sound effects as [`SoundCommand`]s. [`Gfx`] owns the canvas
//! and every texture, and [`Audio`] mixes the embedded sound effects.

mod assets;
mod audio;
mod direction;
mod fruit;
mod game;
mod geometry;
mod gfx;
mod ghost;
mod high_score;
mod maze;
mod pacman;
mod text;
mod timer;

pub use audio::{Audio, Sound, SoundCommand};
pub use direction::Direction;
pub use game::Game;
pub use geometry::{WINDOW_HEIGHT, WINDOW_WIDTH};
pub use gfx::Gfx;
