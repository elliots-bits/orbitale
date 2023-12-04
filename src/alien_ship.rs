use std::{f32::consts::PI, time::Instant};

use bevy::prelude::*;
use bevy_rapier2d::dynamics::{ExternalImpulse, Velocity};

use crate::{
    lasers::{self, LaserAbility},
    player::PlayerMarker,
};

const DRIVE_ENGINE_IMPULSE: f32 = 1.0;
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

    // For now, it is very dumb. It aims at the bad and accelerates if it points in kinda the player direction.

    if let Ok(player_t) = player.get_single() {
        for (entity, t, v, mut laser_ability) in query.iter_mut() {
            let mut linear_impulse = Vec2::ZERO;
            let mut angular_impulse = 0.0;

            let local_forward = t.up().xy();
            let d = (player_t.translation - t.translation).xy();
            let theta = d.y.atan2(d.x);
            let ship_angle = local_forward.y.atan2(local_forward.x);
            if d.length() < MAX_SHOOT_DISTANCE
                && theta.abs() < MAX_SHOOT_THETA
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
            if (theta - ship_angle).abs() < MAX_DRIVE_THETA {
                linear_impulse.x += DRIVE_ENGINE_IMPULSE;
            }

            angular_impulse += (theta - ship_angle).signum() * DRIVE_ENGINE_IMPULSE * ROTATION_MUL;

            commands.entity(entity).insert(ExternalImpulse {
                impulse: local_forward.rotate(linear_impulse),
                torque_impulse: angular_impulse,
            });
        }
    } else {
        error!("Can't process Alien Ships: player not found");
    }
}
