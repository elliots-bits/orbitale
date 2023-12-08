use std::{f32::consts::PI, thread::current};

use bevy::prelude::*;

const STABILIZE_ANGULAR_VELOCITY_THRESHOLD: f32 = 2.0 * 2.0 * PI;

#[derive(Component)]
pub struct OrientationController {
    pub rotation_target: Option<f32>,
    pub torque_available: f32,
}

impl OrientationController {
    pub fn new(torque_available: f32) -> Self {
        Self {
            rotation_target: None,
            torque_available,
        }
    }

    fn to_bounded_positive_angle(a: f32) -> f32 {
        let a = a % (2.0 * PI);
        if a < 0.0 {
            a + 2.0 * PI
        } else {
            a
        }
    }

    pub fn target(&mut self, target: f32) {
        self.rotation_target = Some(OrientationController::to_bounded_positive_angle(target));
    }

    pub fn unset(&mut self) {
        self.rotation_target = None;
    }

    pub fn time_to_target(&self, current_orientation: f32, angular_velocity: f32) -> Option<f32> {
        let current_orientation =
            OrientationController::to_bounded_positive_angle(current_orientation);
        self.rotation_target.map(|target| {
            let sec_per_turn = 2.0 * PI / angular_velocity.abs();
            let arc_to_cover = target - current_orientation;
            let signed_duration = arc_to_cover / angular_velocity;
            if signed_duration < 0.0 {
                signed_duration + sec_per_turn
            } else {
                signed_duration
            }
        })
    }

    pub fn should_brake(&self, current_orientation: f32, angular_velocity: f32) -> Option<bool> {
        let current_orientation =
            OrientationController::to_bounded_positive_angle(current_orientation);
        if self.rotation_target.is_some() {
            let ttt = self
                .time_to_target(current_orientation, angular_velocity)
                .unwrap();
            let reverse_ttt = self
                .time_to_target(current_orientation, -angular_velocity)
                .unwrap();
            debug!("ttt={}", ttt);
            debug!("rev_ttt={}", reverse_ttt);
            if reverse_ttt < ttt && ttt > 0.5 {
                // It's better to reverse rotation.
                Some(true)
            } else {
                // We compute what angular speed we'll have when passing the target if we start braking now
                // by integrating torque applied to current angular velocity during time_to_target.
                let braking_torque = self.torque_available * angular_velocity.signum();
                let speed_at_target_time_if_braking_starts_now =
                    ((angular_velocity * braking_torque * ttt.powf(2.0)) / 2.0)
                        + angular_velocity * ttt
                        + angular_velocity;
                // If this speed is of the opposite sign of the current speed, its too soon to brake.
                // Else, we are overshooting and need to brake now.
                Some(
                    angular_velocity.signum()
                        == speed_at_target_time_if_braking_starts_now.signum(),
                )
            }
        } else {
            None
        }
    }

    pub fn should_stabilize(&self, angular_velocity: f32) -> bool {
        angular_velocity > STABILIZE_ANGULAR_VELOCITY_THRESHOLD
    }

    pub fn torque_needed(&self, current_orientation: f32, angular_velocity: f32) -> f32 {
        let current_orientation =
            OrientationController::to_bounded_positive_angle(current_orientation);
        debug!("theta={}, v={}", current_orientation, angular_velocity);
        debug!("target={}", self.rotation_target.unwrap());

        if self.should_stabilize(angular_velocity) {
            debug!("stabilizing");
            -angular_velocity.signum() * self.torque_available
        } else {
            self.should_brake(current_orientation, angular_velocity)
                .map_or(0.0, |should_brake| {
                    if should_brake {
                        debug!(
                            "braking, torque: {}",
                            self.torque_available * -angular_velocity.signum()
                        );
                        self.torque_available * -angular_velocity.signum()
                    } else {
                        debug!(
                            "accelerating, torque: {}",
                            self.torque_available * angular_velocity.signum()
                        );
                        self.torque_available * angular_velocity.signum()
                    }
                })
        }
    }
}

#[test]
pub fn test_time_to_target() {
    let epsilon = 0.00001;
    let mut controller = OrientationController {
        rotation_target: None,
        torque_available: 0.0,
    };

    controller.target(PI * 17.0);
    assert!((controller.rotation_target.unwrap() - PI).abs() < epsilon);

    let ttt = controller.time_to_target(PI / 2.0, PI).unwrap();
    assert!(ttt > 0.0);
    assert!(ttt - 0.5 < epsilon);

    let ttt = controller.time_to_target(PI / 2.0, -PI).unwrap();
    assert!(ttt > 0.0);
    assert!(ttt - 1.5 < epsilon);

    let ttt = controller.time_to_target(-PI / 2.0, -PI).unwrap();
    assert!(ttt > 0.0);
    assert!(ttt - 0.5 < epsilon);

    let ttt = controller.time_to_target(-PI / 2.0, PI).unwrap();
    assert!(ttt > 0.0);
    assert!(ttt - 1.5 < epsilon);
}
