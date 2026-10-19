use sdl2::Sdl;
use sdl2::audio::{AudioCVT, AudioCallback, AudioDevice, AudioSpecDesired, AudioSpecWAV};
use sdl2::rwops::RWops;

use crate::assets::sounds;

/// Each sound has its own mixer channel, so replaying a sound restarts it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sound {
    Intro,
    EatFruit,
    ExtraLife,
    PacDeath,
    GhostDeath,
    ScatterGhost,
    Waka,
}

impl Sound {
    /// Every sound, in declaration order.
    const ALL: [Self; 7] = [
        Self::Intro,
        Self::EatFruit,
        Self::ExtraLife,
        Self::PacDeath,
        Self::GhostDeath,
        Self::ScatterGhost,
        Self::Waka,
    ];

    fn wav(self) -> &'static [u8] {
        match self {
            Self::Intro => sounds::INTRO,
            Self::EatFruit => sounds::EAT_FRUIT,
            Self::ExtraLife => sounds::EXTRA_LIFE,
            Self::PacDeath => sounds::PAC_DEATH,
            Self::GhostDeath => sounds::GHOST_DEATH,
            Self::ScatterGhost => sounds::SCATTER_GHOST,
            Self::Waka => sounds::WAKA,
        }
    }

    fn loops(self) -> bool {
        matches!(self, Self::ScatterGhost | Self::Waka)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoundCommand {
    Play(Sound),
    Stop(Sound),
}

/// Every channel plays at 10/128 of full volume, as the original set it.
const VOLUME: i32 = 10;
const MAX_VOLUME: i32 = 128;

/// Plays sounds on the default output device, or stays silent if none
/// could be opened.
pub struct Audio {
    device: Option<AudioDevice<Mixer>>,
}

impl Audio {
    pub fn new(sdl: &Sdl) -> Self {
        let device = open_device(sdl)
            .inspect_err(|e| eprintln!("Sound disabled: {e}"))
            .ok();
        Self { device }
    }

    pub fn apply(&mut self, commands: impl IntoIterator<Item = SoundCommand>) {
        let Some(device) = &mut self.device else {
            return;
        };
        let mut mixer = device.lock();
        for command in commands {
            match command {
                SoundCommand::Play(sound) => mixer.positions[sound as usize] = Some(0),
                SoundCommand::Stop(sound) => mixer.positions[sound as usize] = None,
            }
        }
    }
}

fn open_device(sdl: &Sdl) -> Result<AudioDevice<Mixer>, String> {
    let desired = AudioSpecDesired {
        freq: Some(44_100),
        channels: Some(2),
        samples: Some(1024),
    };
    let mut device = sdl
        .audio()?
        .open_playback(None, &desired, |_| Mixer::default())?;
    let spec = device.spec();
    let (format, channels, freq) = (spec.format, spec.channels, spec.freq);
    let mut samples: [Vec<i16>; 7] = Default::default();
    for (sound, samples) in Sound::ALL.into_iter().zip(&mut samples) {
        let wav = AudioSpecWAV::load_wav_rw(&mut RWops::from_bytes(sound.wav())?)?;
        let cvt = AudioCVT::new(wav.format, wav.channels, wav.freq, format, channels, freq)?;
        let bytes = cvt.convert(wav.buffer().to_vec());
        *samples = bytes
            .as_chunks()
            .0
            .iter()
            .map(|&pair| i16::from_ne_bytes(pair))
            .collect();
    }
    device.lock().samples = samples;
    device.resume();
    Ok(device)
}

/// Sums the playing channels, like SDL_mixer does.
#[derive(Default)]
struct Mixer {
    /// Converted interleaved samples of each sound.
    samples: [Vec<i16>; 7],
    /// Playback position of each channel, `None` when silent.
    positions: [Option<usize>; 7],
}

impl AudioCallback for Mixer {
    type Channel = i16;

    fn callback(&mut self, out: &mut [i16]) {
        out.fill(0);
        for ((sound, position), samples) in Sound::ALL
            .into_iter()
            .zip(&mut self.positions)
            .zip(&self.samples)
        {
            let Some(mut index) = *position else {
                continue;
            };
            let mut playing = true;
            for sample in out.iter_mut() {
                if index >= samples.len() {
                    if !sound.loops() || samples.is_empty() {
                        playing = false;
                        break;
                    }
                    index = 0;
                }
                let mixed = i32::from(*sample) + i32::from(samples[index]) * VOLUME / MAX_VOLUME;
                *sample = mixed.clamp(i16::MIN.into(), i16::MAX.into()) as i16;
                index += 1;
            }
            *position = playing.then_some(index);
        }
    }
}
