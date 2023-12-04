use std::{f32::consts::PI, time::Instant};

use bevy::prelude::*;
use bevy_rapier2d::dynamics::{ExternalImpulse, Velocity};

use crate::{
    lasers::{self, LaserAbility},
    player::PlayerMarker,
};

const DRIVE_ENGINE_IMPULSE: f32 = 2.0;
const LASER_KNOCKBACK_IMPULSE: f32 = 20.0;
const ROTATION_MUL: f32 = 25.0;

pub const ALIEN_SHIP_LASER_COOLDOWN_S: f32 = 0.5;

const MAX_SHOOT_DISTANCE: f32 = 500.0;
const MAX_SHOOT_THETA: f32 = PI / 8.0;
const MAX_DRIVE_THETA: f32 = PI / 4.0;

#[derive(Component)]
pub struct AlienShipMarker;

pub fn update(
    mut commands: Commands,
    player: Query<&Transform, With<PlayerMarker>>,
    mut query: Query<(Entity, &Transform, &Velocity, &mut LaserAbility), With<AlienShipMarker>>,
) {
    // The actual AI is going to be a bit tricky.
    // We'll at least have to implement a basic PID control loop.

    // For now, it is very dumb. It aims at the bad and accelerates if it points in kinda in the player direction.

    if let Ok(player_t) = player.get_single() {
        for (entity, t, v, mut laser_ability) in query.iter_mut() {
            let mut linear_impulse = Vec2::ZERO;
            let mut angular_impulse = 0.0;

            let local_forward = t.up().xy();
            let d = (player_t.translation - t.translation).xy();
            let orientation_to_player = local_forward.angle_between(d);

            if d.length() < MAX_SHOOT_DISTANCE
                && orientation_to_player.abs() < MAX_SHOOT_THETA
                && laser_ability.ready()
            {
                let laser_angle = local_forward.y.atan2(local_forward.x);
                lasers::spawn(
                    &mut commands,
                    t.translation.xy() + t.up().xy().normalize() * 32.0,
                    local_forward.rotate(Vec2 { x: 1000.0, y: 0.0 }) + v.linvel,
                    laser_angle,
                    lasers::LaserOrigin::Enemy,
                );
                laser_ability.last_shot = Some(Instant::now());
                linear_impulse.x -= LASER_KNOCKBACK_IMPULSE;
            }
            if orientation_to_player.abs() < MAX_DRIVE_THETA {
                linear_impulse.x += DRIVE_ENGINE_IMPULSE;
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
                if time_to_oriented_s >= 0.25 {
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

            commands.entity(entity).insert(ExternalImpulse {
                impulse: local_forward.rotate(linear_impulse),
                torque_impulse: angular_impulse,
            });
        }
    } else {
        error!("Can't process Alien Ships: player not found");
    }
}
