//! Every asset is compiled into the executable, so the game ships as a
//! single self-contained binary.

pub mod textures {
    pub const MAP: &[u8] = include_bytes!("../assets/textures/Map24.png");
    pub const PELLET: &[u8] = include_bytes!("../assets/textures/Pellet24.png");
    pub const ENERGIZER: &[u8] = include_bytes!("../assets/textures/Energizer24.png");
    pub const DOOR: &[u8] = include_bytes!("../assets/textures/Door.png");
    pub const LIFE: &[u8] = include_bytes!("../assets/textures/Lives32.png");
    pub const PACMAN: &[u8] = include_bytes!("../assets/textures/PacMan32.png");
    pub const PACMAN_DEATH: &[u8] = include_bytes!("../assets/textures/GameOver32.png");
    pub const GHOST_BODY: &[u8] = include_bytes!("../assets/textures/GhostBody32.png");
    pub const GHOST_EYES: &[u8] = include_bytes!("../assets/textures/GhostEyes32.png");
    pub const FRUIT: &[u8] = include_bytes!("../assets/textures/Fruit32.png");
}

pub mod fonts {
    pub const EMULOGIC: &[u8] = include_bytes!("../assets/fonts/emulogic.ttf");
    pub const VP_PIXEL: &[u8] = include_bytes!("../assets/fonts/VpPixel.ttf");
}

pub mod sounds {
    pub const INTRO: &[u8] = include_bytes!("../assets/sounds/Intro.wav");
    pub const EAT_FRUIT: &[u8] = include_bytes!("../assets/sounds/EatFruit.wav");
    pub const EXTRA_LIFE: &[u8] = include_bytes!("../assets/sounds/ExtraLife.wav");
    pub const PAC_DEATH: &[u8] = include_bytes!("../assets/sounds/PacDeath.wav");
    pub const GHOST_DEATH: &[u8] = include_bytes!("../assets/sounds/GhostDeath.wav");
    pub const SCATTER_GHOST: &[u8] = include_bytes!("../assets/sounds/ScatterGhost.wav");
    pub const WAKA: &[u8] = include_bytes!("../assets/sounds/Waka.wav");
}
