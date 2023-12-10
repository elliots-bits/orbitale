use bevy::prelude::*;

use crate::{alien_ship::ALIEN_SHIP_MASS, GLOBAL_IMPULSE_DURATION_MULT};

#[derive(Component)]
pub struct PositionController {
    // It works in one dimension. Higher level AI orchestrates rotation and global behavior.
    pub thrust_available: f32,
    pub current_command: (f32, f32), // thrust, time of stop
}

impl PositionController {
    pub fn new(thrust: f32) -> Self {
        Self {
            thrust_available: thrust,
            current_command: (0.0, 0.0),
        }
    }

    pub fn sleep(&mut self, time: &Time, duration: f32) {
        // debug!("cmd: sleep for {}", duration);
        self.current_command = (0.0, time.elapsed_seconds() + duration);
    }

    pub fn accelerate(&mut self, time: &Time, duration: f32) {
        // debug!("cmd: accelerate for {}", duration);
        self.current_command = (self.thrust_available, time.elapsed_seconds() + duration);
    }

    #[inline]
    pub fn time_to_stop(&self, v0: f32) -> f32 {
        v0 * ALIEN_SHIP_MASS / (self.thrust_available * GLOBAL_IMPULSE_DURATION_MULT)
    }

    pub fn should_brake(&self, d: f32, v0: f32) -> bool {
        if v0.abs() < 0.0001 || d.signum() != v0.signum() {
            // We're going in the wrong way, we should accelerate towards the target.
            // debug!("should_brake: wrong way");
            false
        } else {
            let tts = self.time_to_stop(v0);
            // debug!("Time to stop: {}", tts);
            tts + 2.0 >= d / v0
        }
    }
}
