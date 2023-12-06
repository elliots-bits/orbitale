use std::{collections::HashMap, f32::consts::PI};

use bevy::{prelude::*, hierarchy};
use bevy_rapier2d::{
    dynamics::{Ccd, RigidBody},
    geometry::{ActiveEvents, Collider, ColliderMassProperties},
};

use crate::{camera::game_layer, gravity::AttractingBody};

pub enum CelestialBodyDynamics {
    Static(Vec2),
    CircularOrbit { r: f32, theta: f32, freq: f32 },
}

#[derive(Component)]
pub struct CelestialBodyMarker;

#[derive(Resource)]
pub struct OrbitHierarchy {
    pub roots: Vec<OrbitHierarchyNode>,
}

pub struct OrbitHierarchyNode {
    pub dynamics: CelestialBodyDynamics,
    pub children: Vec<OrbitHierarchyNode>,
}

impl OrbitHierarchyNode {
    pub fn build(dynamics: CelestialBodyDynamics) -> Self {
        Self {
            dynamics,
            children: vec![],
        }
    }

    pub fn with_child(&mut self, dynamics: CelestialBodyDynamics) -> (&OrbitHierarchyNode, Option<CircularOrbitChain>) {
        let child = Self::build(dynamics);
        self.children.push(child);
        // &child
    }
}

#[derive(Component)]
pub struct CircularOrbitChain {
    pub origin: Vec2,
    pub chain: Vec<(f32, f32, f32)>,
}

impl CircularOrbitChain {
    pub fn pos(&self, dt: f32) -> Vec2 {
        let mut pos = self.origin;
        self.chain
            .iter()
            .fold(self.origin, |pos, (r, theta, freq)| {
                pos + Vec2 {
                    x: (theta * freq).cos() * r,
                    y: (theta * freq).sin() * r,
                }
            })
    }
}

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // This is this environment generation.
    let sprite_radius = 585.0;

    let root = OrbitHierarchyNode::build(CelestialBodyDynamics::Static(()))

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

    let child_child_moon_id = commands
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

    commands.insert_resource(OrbitHierarchy {
        roots: vec![OrbitHierarchyNode {
            entity: parent_moon_id,
            children: vec![OrbitHierarchyNode {
                entity: child_moon_id,
                children: vec![OrbitHierarchyNode {
                    entity: child_child_moon_id,
                    children: vec![],
                }],
            }],
        }],
    })
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
