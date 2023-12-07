use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier2d::dynamics::Velocity;

use crate::{
    impulses_aggregator::AddExternalImpulse,
    lasers::{self, Laser, LaserAbility, LaserOrigin},
    player::PlayerMarker,
    thruster::Thruster,
};

const DRIVE_ENGINE_IMPULSE: f32 = 2.0;
const ROTATION_MUL: f32 = 15.0;

pub const ALIEN_SHIP_LASER_COOLDOWN_S: f32 = 0.25;

const MAX_SHOOT_DISTANCE: f32 = 1000.0;
const MAX_SHOOT_THETA: f32 = PI / 8.0;
const MAX_DRIVE_THETA: f32 = PI / 8.0;
const MIN_ROTATION_THETA: f32 = PI / 32.0; // The aliens are unbeatable if they aim directly at the player

/*
Draft outline of the ships' AI:

Every couple of seconds, the ship's trajectory will be simulated.

When the ship is far from the player, its behavior will depend on current the trajectory:
- Crashing into a celestial body -> execute a collision avoidance maneuver
- No crash planned: maneuver to intersect with player (accelerate towards player & minimize encounter distance, then start decelerating at the right time to match velocities when close to the player)

When the ship is near the player, it will ignore the planned trajectory and focus on attacking the player.
This allows the player to trick the enemy into crashing or being slingshot while it's attacking.

*/

#[derive(Component)]
pub struct AlienShipMarker;

#[derive(Component)]
pub struct OrientationController {
    pub rotation_target: Option<f32>,
    pub torque_available: f32,
}

impl OrientationController {
    fn to_bounded_positive_angle(a: f32) -> f32 {
        let a = a % (2.0 * PI);
        if a < 0.0 {
            a + 2.0 * PI
        } else {
            a
        }
    }

    pub fn set(&mut self, target: f32) {
        self.rotation_target = Some(OrientationController::to_bounded_positive_angle(target));
    }

    pub fn unset(&mut self) {
        self.rotation_target = None;
    }

    fn time_to_target(&self, current_orientation: f32, angular_velocity: f32) -> Option<f32> {
        let current_orientation =
            OrientationController::to_bounded_positive_angle(current_orientation);
        self.rotation_target.map(|target| {
            let sec_per_turn = 2.0 * PI / angular_velocity;
            let arc_to_cover = (target - current_orientation);
            let signed_duration = arc_to_cover / angular_velocity;
            if signed_duration < 0.0 {
                signed_duration + sec_per_turn
            } else {
                signed_duration
            }
        })
    }
}

pub fn update(
    mut commands: Commands,
    time: Res<Time>,
    mut impulses: EventWriter<AddExternalImpulse>,
    player: Query<&Transform, With<PlayerMarker>>,
    mut query: Query<
        (
            Entity,
            &Transform,
            &Velocity,
            &mut LaserAbility,
            &mut Thruster,
        ),
        With<AlienShipMarker>,
    >,
) {
    // The actual AI is going to be a bit tricky.
    // We'll at least have to implement a basic PID control loop.

    // For now, it is very dumb. It aims at the player and accelerates if it points in kinda in the player direction.

    if let Ok(player_t) = player.get_single() {
        for (entity, t, v, mut laser_ability, mut thruster) in query.iter_mut() {
            let mut angular_impulse = 0.0;

            let local_forward = t.up().xy();
            let d = (player_t.translation - t.translation).xy();
            let orientation_to_player = local_forward.angle_between(d);

            if d.length() < MAX_SHOOT_DISTANCE
                && orientation_to_player.abs() < MAX_SHOOT_THETA
                && laser_ability.ready(&time)
            {
                let laser_angle = local_forward.y.atan2(local_forward.x);
                lasers::spawn(
                    &mut commands,
                    t.translation.xy()
                        + v.linvel * time.delta_seconds()
                        + t.up().xy().normalize() * 40.0,
                    local_forward.rotate(Vec2 { x: 1500.0, y: 0.0 }) + v.linvel,
                    laser_angle,
                    Laser {
                        origin: LaserOrigin::Enemy,
                        damage: 10.0,
                        shot_at: time.elapsed_seconds(),
                    },
                );
                laser_ability.last_shot = Some(time.elapsed_seconds());
            }
            if orientation_to_player.abs() < MAX_DRIVE_THETA {
                thruster.throttle(time.delta_seconds());
            } else {
                thruster.release(time.delta_seconds());
            }

            // Let's make the aliens dumb. But not TOO dumb.
            let time_to_oriented_s = orientation_to_player / v.angvel;

            let torque_sign = if v.angvel > PI * 2.0 * 2.0 {
                // If we're spinning at more than 2 rotations per second, just stabilize.
                -v.angvel.signum()
            } else {
                // Otherwise, control rotation to aim at the player.
                // This is the same principle as a PID control loop except we don't apply its perfect maths.
                // If we did, the aliens would crush the player with perfect driving.
                // Instead, we approximate an incompetent driver behavior.
                if orientation_to_player.abs() <= MIN_ROTATION_THETA {
                    0.0
                } else if time_to_oriented_s >= 0.25 {
                    // If it'll take more than 0.25 seconds to align with the player at this rate, we'll rotate towards it.
                    orientation_to_player.signum()
                } else if time_to_oriented_s >= 0.1 {
                    // We let it rotate.
                    0.0
                } else if time_to_oriented_s >= 0.0 {
                    // When we'll reach the desired orientation under 0.1s, we brake.
                    -orientation_to_player.signum()
                } else {
                    // We're rotating in the wrong direction and my trigonometry is terrible. Just rotate towards the player.
                    orientation_to_player.signum()
                }
            };

            angular_impulse += torque_sign * DRIVE_ENGINE_IMPULSE * ROTATION_MUL;
            impulses.send(AddExternalImpulse {
                entity,
                impulse: Vec2::ZERO,
                torque_impulse: angular_impulse,
            });
        }
    } else {
        error!("Can't process Alien Ships: player not found");
    }
}

pub fn cleanup(mut commands: Commands, ships: Query<Entity, With<AlienShipMarker>>) {
    for entity in ships.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
