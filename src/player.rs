use std::{f32::consts::PI, time::Instant};

use bevy::prelude::*;
use bevy_rapier2d::{
    dynamics::{Ccd, Damping, ExternalImpulse, RigidBody, Velocity},
    geometry::{Collider, ColliderMassProperties},
};

use crate::lasers;

const DRIVE_ENGINE_IMPULSE: f32 = 2.0;
const BRAKE_ENGINE_IMPULSE: f32 = 1.0;
const ROTATION_MUL: f32 = 15.0;

const LASER_COOLDOWN_S: f32 = 0.25;

#[derive(Component)]
pub struct PlayerMarker;

#[derive(Component)]
pub struct LaserAbility {
    pub last_shot: Option<Instant>,
}

pub fn control(
    mut commands: Commands,
    mut player: Query<(Entity, &mut LaserAbility, &Transform, &Velocity), With<PlayerMarker>>,
    keys: Res<Input<KeyCode>>,
) {
    if let Ok((entity, mut laser_ability, transform, velocity)) = player.get_single_mut() {
        let mut linear = Vec2::ZERO;
        let mut angular = 0.0;
        if keys.pressed(KeyCode::Up) {
            linear.y += DRIVE_ENGINE_IMPULSE;
        }
        if keys.pressed(KeyCode::Down) {
            linear.y -= BRAKE_ENGINE_IMPULSE;
        }
        if keys.pressed(KeyCode::Right) {
            angular -= DRIVE_ENGINE_IMPULSE * ROTATION_MUL;
        }
        if keys.pressed(KeyCode::Left) {
            angular += DRIVE_ENGINE_IMPULSE * ROTATION_MUL;
        }
        let local_forward = transform.right().xy();
        if keys.pressed(KeyCode::Space) {
            let fwd = transform.up().xy();
            let laser_angle = fwd.y.atan2(fwd.x) + PI / 2.0;
            if let Some(last_shot) = laser_ability.last_shot {
                if last_shot.elapsed().as_secs_f32() >= LASER_COOLDOWN_S {
                    lasers::spawn(
                        &mut commands,
                        transform.translation.xy() + transform.up().xy().normalize() * 32.0,
                        Vec2 { x: 0.0, y: 1000.0 }.rotate(local_forward) + velocity.linvel,
                        laser_angle,
                        lasers::LaserOrigin::Player,
                    );
                    laser_ability.last_shot = Some(Instant::now());
                }
            } else {
                lasers::spawn(
                    &mut commands,
                    transform.translation.xy() + transform.up().xy().normalize() * 32.0,
                    Vec2 { x: 0.0, y: 1000.0 }.rotate(local_forward) + velocity.linvel,
                    laser_angle,
                    lasers::LaserOrigin::Player,
                );
                laser_ability.last_shot = Some(Instant::now());
            }
        }
        commands.entity(entity).insert(ExternalImpulse {
            impulse: linear.rotate(local_forward),
            torque_impulse: angular,
        });
    }
}

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        PlayerMarker,
        LaserAbility { last_shot: None },
        SpriteBundle {
            texture: asset_server.load("spaceship_dev1.png"),
            ..default()
        },
        Ccd::enabled(),
        RigidBody::Dynamic,
        Collider::ball(32.0),
        ColliderMassProperties::Mass(1.0),
        Damping {
            linear_damping: 0.02,
            angular_damping: 0.25,
        },
        Velocity::default(),
    ));
}
