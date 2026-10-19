# Rust Pac-Man

A Rust port of PacMan game.

Use the arrow keys or WASD to move. The best score is saved to
`HighScore.txt` in the working directory.

## Build

```sh
cargo run --release
```

By default SDL2 is compiled from source and linked statically, and every
texture, font and sound is embedded, so `target/release/rust_pacman` is a
single file that needs no SDL libraries. This needs CMake and a C compiler.
On Linux, also install the development packages SDL2 needs for video and
audio, such as the X11 or Wayland and ALSA or PulseAudio headers.

To link against an installed SDL2 instead:

```sh
cargo run --release --no-default-features
```

## Layout

- `game.rs`: the round, level and scoring state machine.
- `pacman.rs`, `ghost.rs`, `fruit.rs`: the actors.
- `maze.rs`: the layout, collision and food.
- `gfx.rs`: textures and drawing. `text.rs` renders text exactly like
  SDL_ttf's solid mode, so SDL_ttf is not needed.
- `audio.rs`: a small mixer that plays sounds the way SDL_mixer does, so
  SDL_mixer is not needed.
