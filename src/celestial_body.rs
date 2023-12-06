use bevy::prelude::*;
use bevy_rapier2d::{
    dynamics::{Ccd, RigidBody},
    geometry::{ActiveEvents, Collider, ColliderMassProperties},
};

use crate::{camera::game_layer, gravity::AttractingBody};

#[derive(Component)]
pub struct CelestialBodyMarker;

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // This is this environment generation.
    let radius = 585.0;
    let scale = 2.0;
    let transform =
        Transform::from_translation(Vec3::new(2000.0, 2000.0, 0.0)).with_scale(Vec3::splat(scale));
    commands.spawn((
        CelestialBodyMarker,
        SpriteBundle {
            texture: asset_server.load("mike-petrucci-moon.png"),
            transform,
            ..default()
        },
        Ccd::enabled(),
        RigidBody::Fixed,
        AttractingBody,
        Collider::ball(radius),
        ColliderMassProperties::Mass(1e7),
        ActiveEvents::COLLISION_EVENTS,
        game_layer(),
    ));
}

pub fn update() {}

pub fn cleanup(mut commands: Commands, bodies: Query<Entity, With<CelestialBodyMarker>>) {
    for entity in bodies.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
