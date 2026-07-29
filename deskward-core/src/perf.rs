//! Session performance counters (Faz 4).

use std::time::{Duration, Instant};

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SessionMetrics {
    pub frames_received: u64,
    pub bytes_received: u64,
    pub fps: f64,
    pub decoder: String,
}

#[derive(Debug)]
pub struct SessionMetricsTracker {
    last_tick: Instant,
    frames_since_tick: u32,
    pub frames_received: u64,
    pub bytes_received: u64,
    pub fps: f64,
    decoder: String,
}

impl Default for SessionMetricsTracker {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            last_tick: now,
            frames_since_tick: 0,
            frames_received: 0,
            bytes_received: 0,
            fps: 0.0,
            decoder: "none".into(),
        }
    }
}

impl SessionMetricsTracker {
    pub fn set_decoder(&mut self, name: &str) {
        self.decoder = name.to_string();
    }

    pub fn on_frame(&mut self, byte_len: usize) {
        self.frames_received += 1;
        self.bytes_received += byte_len as u64;
        self.frames_since_tick += 1;
        let elapsed = self.last_tick.elapsed();
        if elapsed >= Duration::from_secs(1) {
            self.fps = self.frames_since_tick as f64 / elapsed.as_secs_f64();
            self.frames_since_tick = 0;
            self.last_tick = Instant::now();
        }
    }

    pub fn snapshot(&self) -> SessionMetrics {
        SessionMetrics {
            frames_received: self.frames_received,
            bytes_received: self.bytes_received,
            fps: self.fps,
            decoder: self.decoder.clone(),
        }
    }
}
