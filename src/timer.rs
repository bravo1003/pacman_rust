/// A stopwatch over the caller's millisecond clock.
#[derive(Clone, Copy, Debug, Default)]
pub struct Timer {
    started_at: Option<u32>,
}

impl Timer {
    pub fn started(now: u32) -> Self {
        Self {
            started_at: Some(now),
        }
    }

    /// Starts the timer, or restarts it if it is already running.
    pub fn start(&mut self, now: u32) {
        self.started_at = Some(now);
    }

    pub fn reset(&mut self) {
        self.started_at = None;
    }

    pub fn is_started(&self) -> bool {
        self.started_at.is_some()
    }

    /// Milliseconds since the timer was started, or 0 when stopped.
    pub fn ticks(&self, now: u32) -> u32 {
        self.started_at.map_or(0, |start| now.wrapping_sub(start))
    }
}
