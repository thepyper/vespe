// src/screens/splash_screen.rs
use std::time::{Instant, Duration};

pub struct SplashScreen {
    start_time: Instant,
    animation_frame: usize,
    // Add ASCII art frames here later
}

impl SplashScreen {
    pub fn new() -> Self {
        SplashScreen {
            start_time: Instant::now(),
            animation_frame: 0,
        }
    }

    pub fn is_finished(&self) -> bool {
        self.start_time.elapsed() >= Duration::from_secs(5)
    }

    pub fn update_animation(&mut self) {
        // Logic to update animation_frame based on time or ticks
        // For now, a simple increment
        self.animation_frame = (self.animation_frame + 1) % 4; // Assuming 4 frames for now
    }

    pub fn get_current_frame(&self) -> usize {
        self.animation_frame
    }
}
