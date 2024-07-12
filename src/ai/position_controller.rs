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
        self.current_command = (0.0, time.elapsed_seconds() + duration);
    }

    pub fn accelerate(&mut self, time: &Time, duration: f32) {
        self.current_command = (self.thrust_available, time.elapsed_seconds() + duration);
    }

    #[inline]
    pub fn time_to_stop(&self, v0: f32) -> f32 {
        v0 * ALIEN_SHIP_MASS / (self.thrust_available * GLOBAL_IMPULSE_DURATION_MULT)
    }

    pub fn should_brake(&self, distance: f32, velocity: f32) -> bool {
        if velocity.abs() < 0.0001 || distance.signum() != velocity.signum() {
            // We're going in the wrong way, we should accelerate towards the target.
            false
        } else {
            // We start braking if the time needed to stop is greater than the time to reach the target at the current speed.
            // We add a 1 second margin to execute the turn-around maneuver.
            self.time_to_stop(velocity) + 1.0 >= distance / velocity
        }
    }
}
