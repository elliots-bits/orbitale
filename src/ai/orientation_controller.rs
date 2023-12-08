use std::f32::consts::PI;

use bevy::prelude::*;

const SHIP_ANGULAR_INERTIA: f32 = 0.5 * 1.0 * 32.0; // 0.5 * mass * radius
const STABILIZE_ANGULAR_VELOCITY_THRESHOLD: f32 = 2.0 * 2.0 * PI;
const MIN_ROTATION_THETA: f32 = PI / 32.0; // We're close enough.
const MAX_CONTROLLER_UPDATES_PER_FRAME: u32 = 5;
const STALE_CONTROLLER_COMMAND_AGE: f32 = 0.1;

#[derive(Component)]
pub struct OrientationController {
    pub rotation_target: Option<f32>,
    pub torque_available: f32,
    pub current_requested_torque: f32,
    pub updated_at: f32,
}

impl OrientationController {
    pub fn new(torque_available: f32) -> Self {
        Self {
            rotation_target: None,
            current_requested_torque: 0.0,
            torque_available,
            updated_at: 0.0,
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

    pub fn time_to_target(&self, p0: f32, v0: f32, a: f32) -> Option<f32> {
        let a = a / SHIP_ANGULAR_INERTIA;
        let p0 = OrientationController::to_bounded_positive_angle(p0);
        self.rotation_target.map(|target| {
            let arc = -(target - p0);
            let arc = if arc < -PI {
                arc + 2.0 * PI
            } else if arc > PI {
                arc - 2.0 * PI
            } else {
                arc
            };
            // debug!("arc_to_target: arc={}", arc);
            let sec_per_turn = 2.0 * PI / v0.abs();
            if (a - 0.0).abs() < 0.001 {
                if (v0 - 0.0).abs() < 0.001 {
                    f32::INFINITY
                } else {
                    // debug!("arc_to_target: no torque ");
                    let signed_duration = -arc / v0;
                    // debug!("arc_to_target: signed_duration={}", signed_duration);
                    // debug!("arc_to_target: second_per_turn={}", sec_per_turn);
                    if signed_duration < 0.0 {
                        signed_duration + sec_per_turn
                    } else {
                        signed_duration
                    }
                }
            } else {
                let det = v0.powf(2.0) - 4.0 * a * arc;
                if det < 0.0 {
                    debug!("Would rotate in wrong direction");
                    let arc = if arc < 0.0 {
                        arc + PI * 2.0
                    } else {
                        arc - PI * 2.0
                    };
                    let x1 = (-v0 - (v0.powf(2.0) - 2.0 * a * arc).sqrt()) / (2.0 * a);
                    let x2 = (-v0 + (v0.powf(2.0) - 2.0 * a * arc).sqrt()) / (2.0 * a);
                    debug!("x1={}, x2={}", x1, x2);
                    x2.abs()
                } else if det == 0.0 {
                    -v0 / (2.0 * a)
                } else {
                    let x1 = (-v0 - (v0.powf(2.0) - 2.0 * a * arc).sqrt()) / (2.0 * a);
                    let x2 = (-v0 + (v0.powf(2.0) - 2.0 * a * arc).sqrt()) / (2.0 * a);
                    debug!("x1={}, x2={}", x1, x2);
                    x2.abs()
                }
            }
        })
    }

    pub fn should_brake(&self, current_orientation: f32, angular_velocity: f32) -> Option<bool> {
        let current_orientation =
            OrientationController::to_bounded_positive_angle(current_orientation);
        if self.rotation_target.is_some() {
            // We compute what angular speed we'll have when passing the target if we start braking now
            // by integrating torque applied to current angular velocity during time_to_target.
            let ttt = self
                .time_to_target(current_orientation, angular_velocity, 0.0)
                .unwrap();
            // debug!("should_brake: ttt={}", ttt);
            let braking_torque =
                self.torque_available * -angular_velocity.signum() / SHIP_ANGULAR_INERTIA;
            let speed_at_target_time_if_braking_starts_now =
                angular_velocity + braking_torque * ttt;
            // debug!(
            //     "should_brake: speed_at_target={}",
            //     speed_at_target_time_if_braking_starts_now
            // );
            // If this speed is of the opposite sign of the current speed, its too soon to brake.
            // Else, we are overshooting and need to brake now.
            Some(angular_velocity.signum() == speed_at_target_time_if_braking_starts_now.signum())
        } else {
            None
        }
    }

    pub fn should_stabilize(&self, angular_velocity: f32) -> bool {
        angular_velocity.abs() > STABILIZE_ANGULAR_VELOCITY_THRESHOLD
    }

    pub fn torque_needed(&self, current_orientation: f32, angular_velocity: f32) -> f32 {
        let current_orientation =
            OrientationController::to_bounded_positive_angle(current_orientation);
        // debug!("theta={}, v={}", current_orientation, angular_velocity);
        // debug!("target={}", self.rotation_target.unwrap());

        if self.should_stabilize(angular_velocity) {
            // debug!("stabilizing");
            -angular_velocity.signum() * self.torque_available
        } else if (current_orientation - self.rotation_target.unwrap()).abs() < MIN_ROTATION_THETA {
            // debug!("target reached");
            0.0
        } else {
            let ttt_at_current_speed = self
                .time_to_target(current_orientation, angular_velocity, 0.0)
                .unwrap();
            // debug!(
            // "At current speed, target will be reached in: {}",
            //     ttt_at_current_speed
            // );
            if ttt_at_current_speed > 1.0 {
                // debug!("Analyzing situation");
                let positive_ttt = self
                    .time_to_target(current_orientation, angular_velocity, self.torque_available)
                    .unwrap();
                let negative_ttt = self
                    .time_to_target(
                        current_orientation,
                        angular_velocity,
                        -self.torque_available,
                    )
                    .unwrap();
                // debug!("Clockwise: {}", negative_ttt);
                // debug!("Counterclockwise: {}", positive_ttt);
                if positive_ttt < negative_ttt {
                    // debug!("Applying torque {}", self.torque_available);
                    self.torque_available
                } else {
                    // debug!("Applying torque: {}", -self.torque_available);
                    -self.torque_available
                }
            } else {
                // debug!("Executing maneuver");
                let should_brake = self
                    .should_brake(current_orientation, angular_velocity)
                    .unwrap();
                if should_brake {
                    // debug!(
                    //     "braking, torque: {}",
                    //     self.torque_available * -angular_velocity.signum()
                    // );
                    self.torque_available * -angular_velocity.signum()
                } else if ttt_at_current_speed > 0.1 {
                    // debug!(
                    //     "accelerating, torque: {}",
                    //     self.torque_available * angular_velocity.signum()
                    // );
                    self.torque_available * angular_velocity.signum()
                } else {
                    // debug!("keep rotating..");
                    0.0
                }
            }
        }
    }
}

#[test]
pub fn test_time_to_target() {
    let epsilon = 0.00001;
    let mut controller = OrientationController {
        rotation_target: None,
        torque_available: 0.0,
        current_requested_torque: 0.0,
        updated_at: 0.0,
    };

    controller.target(PI * 17.0);
    assert!((controller.rotation_target.unwrap() - PI).abs() < epsilon);

    let ttt = controller.time_to_target(PI / 2.0, PI, 0.0).unwrap();
    assert!(ttt > 0.0);
    assert!(ttt - 0.5 < epsilon);

    let ttt = controller.time_to_target(PI / 2.0, -PI, 0.0).unwrap();
    assert!(ttt > 0.0);
    assert!(ttt - 1.5 < epsilon);

    let ttt = controller.time_to_target(-PI / 2.0, -PI, 0.0).unwrap();
    assert!(ttt > 0.0);
    assert!(ttt - 0.5 < epsilon);

    let ttt = controller.time_to_target(-PI / 2.0, PI, 0.0).unwrap();
    assert!(ttt > 0.0);
    assert!(ttt - 1.5 < epsilon);

    let ttt = controller.time_to_target(PI / 2.0, 0.0, 0.0).unwrap();
    assert_eq!(ttt, f32::INFINITY);

    let ttt = controller.time_to_target(PI / 2.0, 0.0, PI).unwrap();
    assert!(ttt > 0.0);
    assert!(ttt - 1.0 < epsilon);

    let ttt = controller.time_to_target(PI / 2.0, 0.0, -PI).unwrap();
    assert!(ttt > 0.0);
    assert!(ttt - 0.8660253 < epsilon);
}
