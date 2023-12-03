use bevy::prelude::*;
use bevy_rapier2d::{
    dynamics::{Ccd, Damping, ExternalImpulse, RigidBody, Velocity},
    geometry::{Collider, ColliderMassProperties},
};

const DRIVE_ENGINE_IMPULSE: f32 = 2.0;
const BRAKE_ENGINE_IMPULSE: f32 = 1.0;
const ROTATION_MUL: f32 = 15.0;

#[derive(Component)]
pub struct PlayerMarker;

pub fn control(
    mut commands: Commands,
    player: Query<(Entity, &Transform), With<PlayerMarker>>,
    keys: Res<Input<KeyCode>>,
) {
    if let Ok((entity, transform)) = player.get_single() {
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
        commands.entity(entity).insert(ExternalImpulse {
            impulse: linear.rotate(transform.right().xy()),
            torque_impulse: angular,
        });
    }
}

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        PlayerMarker,
        SpriteBundle {
            texture: asset_server.load("spaceship_dev1.png"),
            ..default()
        },
        Ccd::enabled(),
        RigidBody::Dynamic,
        Collider::ball(32.0),
        ColliderMassProperties::Mass(1.0),
        Damping {
            linear_damping: 0.0,
            angular_damping: 1.0,
        },
        Velocity::default(),
    ));
}
