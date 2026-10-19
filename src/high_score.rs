use std::fs;
use std::path::PathBuf;

/// The best score so far, kept in a plain text file.
pub struct HighScore {
    path: PathBuf,
    best: u32,
}

impl HighScore {
    /// Reads the saved score; a missing or unreadable file counts as 0.
    pub fn load(path: PathBuf) -> Self {
        let best = fs::read_to_string(&path)
            .ok()
            .and_then(|text| {
                let text = text.trim_start();
                let digits = text
                    .find(|c: char| !c.is_ascii_digit())
                    .unwrap_or(text.len());
                text[..digits].parse().ok()
            })
            .unwrap_or(0);
        Self { path, best }
    }

    pub fn best(&self) -> u32 {
        self.best
    }

    /// Saves `score` if it beats the best one.
    pub fn record(&mut self, score: u32) {
        if score > self.best {
            self.best = score;
            // Saving is best effort; failing to save must not stop the game.
            let _ = fs::write(&self.path, score.to_string());
        }
    }
}
