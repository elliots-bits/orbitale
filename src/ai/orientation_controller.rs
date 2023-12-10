use std::f32::consts::PI;

use bevy::prelude::*;

use crate::{alien_ship::ALIEN_SHIP_MASS, GLOBAL_IMPULSE_DURATION_MULT};

const ANGULAR_INERTIA: f32 = 0.5 * ALIEN_SHIP_MASS * 32.0 * 32.0;
const STABILIZE_ANGULAR_VELOCITY_THRESHOLD: f32 = 4.0 * 2.0 * PI;
const MIN_ROTATION_THETA: f32 = PI / 24.0; // We're close enough.

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
            let (torque, dt) = self.torque_needed(p0, v0);
            self.current_command = (torque, time.elapsed_seconds() + dt);
            // debug!("cmd={:?}", self.current_command);
        }
    }

    pub fn at_target(&self, p0: f32, epsilon: f32) -> bool {
        self.shortest_arc(self.rotation_target.unwrap(), p0).abs() < epsilon
    }

    fn to_bounded_positive_angle(a: f32) -> f32 {
        let a = a % (2.0 * PI);
        if a < 0.0 {
            a + 2.0 * PI
        } else {
            a
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
            Some(false)
        } else {
            let current_orientation =
                OrientationController::to_bounded_positive_angle(current_orientation);
            if self.rotation_target.is_some() {
                let tts = self.time_to_stop(angular_velocity);
                // debug!(
                //     "current={}, target={}, v={}, tts={}",
                //     current_orientation,
                //     self.rotation_target.unwrap(),
                //     angular_velocity,
                //     tts,
                // );
                let ttt_at_current_vel = {
                    let ttt =
                        (self.rotation_target.unwrap() - current_orientation) / angular_velocity;
                    if ttt < 0.0 {
                        ttt + 2.0 * PI / angular_velocity
                    } else {
                        ttt
                    }
                };
                Some(tts + 0.1 >= ttt_at_current_vel)
            } else {
                None
            }
        }
    }

    pub fn should_stabilize(&self, angular_velocity: f32) -> bool {
        angular_velocity.abs() > STABILIZE_ANGULAR_VELOCITY_THRESHOLD
    }

    pub fn torque_needed(&self, current_orientation: f32, angular_velocity: f32) -> (f32, f32) {
        let current_orientation =
            OrientationController::to_bounded_positive_angle(current_orientation);
        // debug!("theta={}, v={}", current_orientation, angular_velocity);
        // debug!("target={}", self.rotation_target.unwrap());

        let shortest_arc = self.shortest_arc(current_orientation, self.rotation_target.unwrap());
        if self.should_stabilize(angular_velocity) {
            // debug!("stabilizing");
            let torque = -angular_velocity.signum() * self.torque_available;
            (torque, 0.1)
        } else if shortest_arc.abs() < MIN_ROTATION_THETA {
            // debug!("target reached");
            (
                0.0,
                (0.45 * (MIN_ROTATION_THETA - shortest_arc.abs()) / angular_velocity.abs())
                    .min(0.25)
                    .max(0.1),
            )
        } else {
            let ttt_at_current_speed = if angular_velocity.abs() == 0.0 {
                f32::INFINITY
            } else {
                let ttt_at_current_speed =
                    (self.rotation_target.unwrap() - current_orientation) / angular_velocity;
                if ttt_at_current_speed < 0.0 {
                    ttt_at_current_speed + 2.0 * PI / angular_velocity
                } else {
                    ttt_at_current_speed
                }
            };

            // debug!(
            //     "At current speed, target will be reached in: {}",
            //     ttt_at_current_speed
            // );
            if ttt_at_current_speed > 1.0 {
                // debug!("we far from target, shortest_arc={}", shortest_arc);
                (
                    self.torque_available * shortest_arc.signum(),
                    (2.1 * angular_velocity.abs() / self.torque_available)
                        .max(0.1)
                        .min(0.33),
                )
            } else {
                // debug!("Executing maneuver");
                let should_brake = self
                    .should_brake(current_orientation, angular_velocity)
                    .unwrap();
                let tts: f32 = self.time_to_stop(angular_velocity);
                if should_brake {
                    // debug!(
                    //     "braking, torque: {}",
                    //     self.torque_available * -angular_velocity.signum()
                    // );
                    (
                        self.torque_available * -angular_velocity.signum(),
                        (tts - 0.05).max(0.05),
                    )
                } else if ttt_at_current_speed > 0.5 {
                    // debug!(
                    //     "accelerating, torque: {}",
                    //     self.torque_available * angular_velocity.signum()
                    // );
                    (
                        self.torque_available * angular_velocity.signum(),
                        (tts - 0.02).max(0.02),
                    )
                } else {
                    // debug!("keep rotating..");
                    (0.0, 0.1)
                }
            }
        }
    }
}
