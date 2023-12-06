use std::{collections::HashMap, f32::consts::PI};

use bevy::prelude::*;
use bevy_rapier2d::{
    dynamics::{Ccd, RigidBody},
    geometry::{ActiveEvents, Collider, ColliderMassProperties},
};

use crate::{camera::game_layer, gravity::AttractingBody};

#[derive(Component)]
pub struct CelestialBodyMarker;

#[derive(Component)]
pub struct FixedOrbit {
    pub parent: Entity,
    pub radius: f32,
    pub theta: f32,
    pub period: f32,
}

impl FixedOrbit {
    pub fn update(&mut self, dt: f32) {
        self.theta += (2.0 * PI * dt) / self.period;
        self.theta %= 2.0 * PI;
    }

    pub fn pos(&self) -> Vec2 {
        Vec2::new(
            self.theta.cos() * self.radius,
            self.theta.sin() * self.radius,
        )
    }
}

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // This is this environment generation.
    let radius = 585.0;

    // let sun_id = commands
    //     .spawn((
    //         CelestialBodyMarker,
    //         SpriteBundle {
    //             texture: asset_server.load("mike-petrucci-moon.png"),
    //             transform: Transform::from_translation(Vec3::new(-10000.0, 100000.0, 0.0))
    //                 .with_scale(Vec3::splat(50.0)),
    //             ..default()
    //         },
    //         Ccd::enabled(),
    //         RigidBody::Fixed,
    //         AttractingBody,
    //         Collider::ball(radius),
    //         ColliderMassProperties::Mass(5e8),
    //         ActiveEvents::COLLISION_EVENTS,
    //         game_layer(),
    //     ))
    //     .id();

    let parent_moon_id = commands
        .spawn((
            CelestialBodyMarker,
            SpriteBundle {
                texture: asset_server.load("mike-petrucci-moon.png"),
                transform: Transform::from_translation(Vec3::new(10000.0, 10000.0, 0.0))
                    .with_scale(Vec3::splat(5.0)),
                ..default()
            },
            // FixedOrbit {
            //     parent: sun_id,
            //     radius: 140000.0,
            //     theta: 0.0,
            //     period: 3000.0,
            // },
            Ccd::enabled(),
            RigidBody::Fixed,
            AttractingBody,
            Collider::ball(radius),
            ColliderMassProperties::Mass(5e7),
            ActiveEvents::COLLISION_EVENTS,
            game_layer(),
        ))
        .id();

    let child_moon_id = commands
        .spawn((
            CelestialBodyMarker,
            SpriteBundle {
                texture: asset_server.load("mike-petrucci-moon.png"),
                transform: Transform::from_scale(Vec3::splat(1.5)),
                ..default()
            },
            FixedOrbit {
                parent: parent_moon_id,
                radius: 18000.0,
                theta: 0.0,
                period: 300.0,
            },
            Ccd::enabled(),
            RigidBody::Fixed,
            AttractingBody,
            Collider::ball(radius),
            ColliderMassProperties::Mass(1e7),
            ActiveEvents::COLLISION_EVENTS,
            game_layer(),
        ))
        .id();

    let _child_child_moon_id = commands
        .spawn((
            CelestialBodyMarker,
            FixedOrbit {
                parent: child_moon_id,
                radius: 7000.0,
                theta: 0.0,
                period: 40.0,
            },
            SpriteBundle {
                texture: asset_server.load("mike-petrucci-moon.png"),
                transform: Transform::from_scale(Vec3::splat(0.5)),
                ..default()
            },
            Ccd::enabled(),
            RigidBody::Fixed,
            AttractingBody,
            Collider::ball(radius),
            ColliderMassProperties::Mass(1.5e6),
            ActiveEvents::COLLISION_EVENTS,
            game_layer(),
        ))
        .id();
}

pub fn update(
    time: Res<Time>,
    mut bodies: Query<(Entity, &mut Transform, Option<&mut FixedOrbit>), With<CelestialBodyMarker>>,
) {
    let mut positions = HashMap::<Entity, Vec2>::new();
    for (e, t, _) in bodies.iter() {
        positions.insert(e, t.translation.xy());
    }

    for (_, mut transform, orbit) in bodies.iter_mut() {
        if let Some(mut orbit) = orbit {
            orbit.update(time.delta_seconds());
            let parent_pos = positions.get(&orbit.parent).unwrap();
            transform.translation = (*parent_pos + orbit.pos()).extend(0.0);
        }
    }
}

pub fn cleanup(mut commands: Commands, bodies: Query<Entity, With<CelestialBodyMarker>>) {
    for entity in bodies.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
