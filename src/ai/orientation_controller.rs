use std::f32::consts::PI;

use bevy::prelude::*;

use crate::{alien_ship::ALIEN_SHIP_MASS, GLOBAL_IMPULSE_DURATION_MULT};

const ANGULAR_INERTIA: f32 = 0.5 * ALIEN_SHIP_MASS * 32.0 * 32.0;
const STABILIZE_ANGULAR_VELOCITY_THRESHOLD: f32 = 4.0 * 2.0 * PI;
pub const MIN_ROTATION_THETA: f32 = PI / 16.0; // We're close enough.

#[derive(Component)]
pub struct OrientationController {
    pub rotation_target: Option<f32>,
    pub torque_available: f32,
    pub current_command: (f32, f32), // torque, time of stop
}

impl OrientationController {
    pub fn new(torque_available: f32) -> Self {
        Self {
            rotation_target: None,
            current_command: (0.0, 0.0),
            torque_available,
        }
    }

    pub fn update_command(&mut self, time: &Time, p0: f32, v0: f32) {
        if self.rotation_target.is_some() {
            let (torque, delta_time) = self.torque_needed(p0, v0);
            self.current_command = (torque, time.elapsed_seconds() + delta_time);
            // debug!("cmd={:?}", self.current_command);
        }
    }

    pub fn at_target(&self, p0: f32, epsilon: f32) -> bool {
        self.shortest_arc(self.rotation_target.unwrap(), p0).abs() < epsilon
    }

    fn to_bounded_positive_angle(theta: f32) -> f32 {
        let theta = theta % (2.0 * PI);
        if theta < 0.0 {
            theta + 2.0 * PI
        } else {
            theta
        }
    }

    fn shortest_arc(&self, a: f32, b: f32) -> f32 {
        let a = OrientationController::to_bounded_positive_angle(a);
        let b = OrientationController::to_bounded_positive_angle(b);
        let arc = -(a - b);
        if arc < -PI {
            arc + 2.0 * PI
        } else if arc > PI {
            arc - 2.0 * PI
        } else {
            arc
        }
    }

    pub fn target(&mut self, target: f32) {
        self.rotation_target = Some(OrientationController::to_bounded_positive_angle(target));
    }

    #[inline]
    pub fn time_to_stop(&self, v0: f32) -> f32 {
        v0.abs() * ANGULAR_INERTIA / (self.torque_available * GLOBAL_IMPULSE_DURATION_MULT)
    }

    pub fn should_brake(&self, current_orientation: f32, angular_velocity: f32) -> Option<bool> {
        if angular_velocity.abs() == 0.0 {
            // If we are not rotating, we don't need to brake.
            Some(false)
        } else {
            let current_orientation =
                OrientationController::to_bounded_positive_angle(current_orientation);
            if self.rotation_target.is_some() {
                // We brake if the time needed to stop matches the time to reach the target at the current velocity
                let time_to_stop = self.time_to_stop(angular_velocity);

                let time_to_target_at_current_velocity = {
                    let time_to_target =
                        (self.rotation_target.unwrap() - current_orientation) / angular_velocity;
                    if time_to_target < 0.0 {
                        time_to_target + 2.0 * PI / angular_velocity.abs()
                    } else {
                        time_to_target
                    }
                };
                Some(time_to_stop + 0.1 >= time_to_target_at_current_velocity)
            } else {
                None
            }
        }
    }

    pub fn should_stabilize(&self, angular_velocity: f32) -> bool {
        angular_velocity.abs() > STABILIZE_ANGULAR_VELOCITY_THRESHOLD
    }

    pub fn torque_needed(&self, current_orientation: f32, angular_velocity: f32) -> (f32, f32) {
        // Compute what torque is needed to reach target orientation using current orientation & angular velocity
        let current_orientation =
            OrientationController::to_bounded_positive_angle(current_orientation);

        let shortest_arc = self.shortest_arc(current_orientation, self.rotation_target.unwrap());
        let arc_torque_limit = shortest_arc.abs().min(1.0);
        if self.should_stabilize(angular_velocity) {
            // We are rotating too fast and need to stabilize.
            // We rotate in the opposite direction of current rotation for the required duration to reach a stop.
            let torque = -angular_velocity.signum() * self.torque_available;
            (torque, 0.1)
        } else if shortest_arc.abs() < MIN_ROTATION_THETA {
            // We are facing the correct orientation.
            // We issue an empty command for the duration that we will be facing the correct orientation.
            (
                0.0,
                (0.45 * (MIN_ROTATION_THETA - shortest_arc.abs()) / angular_velocity.abs())
                    .min(0.25)
                    .max(0.1),
            )
        } else {
            // We are not facing the correct orientation.
            // We need to either accelerate in the required direction or brake to reach a stop at the wanted orientation.
            // We compute the time to reach the target orientation at current velocity and make a decision based on that.
            let time_to_target_at_current_speed = if angular_velocity.abs() == 0.0 {
                f32::INFINITY
            } else {
                let time_to_target_at_current_speed =
                    (self.rotation_target.unwrap() - current_orientation) / angular_velocity;
                if time_to_target_at_current_speed < 0.0 {
                    time_to_target_at_current_speed + 2.0 * PI / angular_velocity.abs()
                } else {
                    time_to_target_at_current_speed
                }
            };

            if time_to_target_at_current_speed > 1.0 {
                // We will reach the target in more that 1 second, we can accelerate.
                (
                    self.torque_available * shortest_arc.signum(),
                    (2.1 * angular_velocity.abs() / self.torque_available)
                        .max(0.05)
                        .min(0.2),
                )
            } else {
                // We will reach the target in less that 1 second. We check if we need to start braking.
                let should_brake = self
                    .should_brake(current_orientation, angular_velocity)
                    .unwrap();
                let time_to_stop: f32 = self.time_to_stop(angular_velocity);
                if should_brake {
                    // We rotate in the opposite direction of current rotation for the duration needed to stop.
                    (
                        self.torque_available * arc_torque_limit * -angular_velocity.signum(),
                        (time_to_stop - 0.05).max(0.01),
                    )
                } else if time_to_target_at_current_speed > 0.5 {
                    // We are more than 0.5 seconds away, we can accelerate for a little bit.
                    (
                        self.torque_available * arc_torque_limit * angular_velocity.signum(),
                        0.01,
                    )
                } else {
                    // We'll brake soon but not right now. We keep rotating.
                    (0.0, 0.05)
                }
            }
        }
    }
}
