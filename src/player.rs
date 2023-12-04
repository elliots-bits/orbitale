use std::time::{Duration, Instant};

use bevy::prelude::*;
use bevy_rapier2d::{
    dynamics::{Ccd, Damping, ExternalImpulse, RigidBody, Velocity},
    geometry::{Collider, ColliderMassProperties},
};

use crate::lasers::{self, LaserAbility};

const DRIVE_ENGINE_IMPULSE: f32 = 4.0;
const BRAKE_ENGINE_IMPULSE: f32 = 2.0;
const LASER_KNOCKBACK_IMPULSE: f32 = 10.0;
const ROTATION_MUL: f32 = 8.0;

const LASER_COOLDOWN_S: f32 = 0.025;

#[derive(Component)]
pub struct PlayerMarker;

pub fn control(
    mut commands: Commands,
    mut player: Query<(Entity, &mut LaserAbility, &Transform, &Velocity), With<PlayerMarker>>,
    keys: Res<Input<KeyCode>>,
) {
    if let Ok((entity, mut laser_ability, transform, velocity)) = player.get_single_mut() {
        let mut linear = Vec2::ZERO;
        let mut angular = 0.0;
        if keys.pressed(KeyCode::Up) {
            linear.x += DRIVE_ENGINE_IMPULSE;
        }
        if keys.pressed(KeyCode::Down) {
            linear.x -= BRAKE_ENGINE_IMPULSE;
        }
        if keys.pressed(KeyCode::Right) {
            angular -= DRIVE_ENGINE_IMPULSE * ROTATION_MUL;
        }
        if keys.pressed(KeyCode::Left) {
            angular += DRIVE_ENGINE_IMPULSE * ROTATION_MUL;
        }
        let local_forward = transform.up().xy();
        if keys.pressed(KeyCode::Space) && laser_ability.ready() {
            let laser_angle = local_forward.y.atan2(local_forward.x);
            lasers::spawn(
                &mut commands,
                transform.translation.xy() + transform.up().xy().normalize() * 32.0,
                Vec2 { x: 2000.0, y: 0.0 }.rotate(local_forward) + velocity.linvel,
                laser_angle,
                lasers::LaserOrigin::Player,
            );
            laser_ability.last_shot = Some(Instant::now());
            linear.x -= LASER_KNOCKBACK_IMPULSE;
        }

        commands.entity(entity).insert(ExternalImpulse {
            impulse: local_forward.rotate(linear),
            torque_impulse: angular,
        });
    }
}

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        PlayerMarker,
        LaserAbility {
            last_shot: None,
            cooldown: Duration::from_secs_f32(LASER_COOLDOWN_S),
        },
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
