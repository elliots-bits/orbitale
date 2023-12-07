use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier2d::{
    dynamics::{Ccd, RigidBody},
    geometry::{ActiveEvents, Collider, ColliderMassProperties},
};

use crate::{camera::game_layer, gravity::AttractingBody};

#[derive(Component)]
pub struct CelestialBodyMarker;

#[derive(Resource)]
pub struct OrbitHierarchy {
    pub roots: Vec<OrbitHierarchyNode>,
}

pub struct OrbitHierarchyNode {
    pub dynamics: CircularOrbitChain,
    pub children: Vec<OrbitHierarchyNode>,
}

impl OrbitHierarchyNode {
    pub fn start(pos: Vec2) -> Self {
        Self {
            dynamics: CircularOrbitChain {
                origin: pos,
                chain: vec![],
            },
            children: vec![],
        }
    }

    pub fn with_child(&mut self, orbit: CircularOrbitDef) -> &mut OrbitHierarchyNode {
        let mut chain = self.dynamics.chain.clone();
        chain.push(orbit);
        let child = OrbitHierarchyNode {
            dynamics: CircularOrbitChain {
                origin: self.dynamics.origin,
                chain,
            },
            children: vec![],
        };
        self.children.push(child);
        self.children.last_mut().unwrap()
    }
}

#[derive(Copy, Clone)]
pub struct CircularOrbitDef {
    theta: f32,
    radius: f32,
    freq: f32,
}

#[derive(Component, Clone)]
pub struct CircularOrbitChain {
    pub origin: Vec2,
    pub chain: Vec<CircularOrbitDef>,
}

impl CircularOrbitChain {
    pub fn update(&mut self, dt: f32) {
        for orbit in self.chain.iter_mut() {
            orbit.theta = (orbit.theta + (dt * orbit.freq)) % (PI * 2.0);
        }
    }

    pub fn pos(&self, dt: f32) -> Vec2 {
        self.chain.iter().fold(
            self.origin,
            |pos,
             CircularOrbitDef {
                 radius,
                 theta,
                 freq,
             }| {
                pos + Vec2 {
                    x: (theta + dt * freq).cos() * radius,
                    y: (theta + dt * freq).sin() * radius,
                }
            },
        )
    }
}

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // This is this environment generation.
    let sprite_radius = 585.0;

    let mut parent_moon_node = OrbitHierarchyNode::start(Vec2 {
        x: 10000.0,
        y: 10000.0,
    });
    commands.spawn((
        CelestialBodyMarker,
        parent_moon_node.dynamics.clone(),
        SpriteBundle {
            texture: asset_server.load("mike-petrucci-moon.png"),
            transform: Transform::default().with_scale(Vec3::splat(5.0)),
            ..default()
        },
        Ccd::enabled(),
        RigidBody::Fixed,
        AttractingBody,
        Collider::ball(sprite_radius),
        ColliderMassProperties::Mass(5e7),
        ActiveEvents::COLLISION_EVENTS,
        game_layer(),
    ));

    let child_moon_node = parent_moon_node.with_child(CircularOrbitDef {
        theta: 0.0,
        radius: 18000.0,
        freq: 1.0 / 300.0,
    });
    commands.spawn((
        CelestialBodyMarker,
        SpriteBundle {
            texture: asset_server.load("mike-petrucci-moon.png"),
            transform: Transform::from_scale(Vec3::splat(1.5)),
            ..default()
        },
        child_moon_node.dynamics.clone(),
        Ccd::enabled(),
        RigidBody::Fixed,
        AttractingBody,
        Collider::ball(sprite_radius),
        ColliderMassProperties::Mass(1e7),
        ActiveEvents::COLLISION_EVENTS,
        game_layer(),
    ));

    let child_child_moon_node = child_moon_node.with_child(CircularOrbitDef {
        theta: 0.0,
        radius: 7000.0,
        freq: 1.0 / 40.0,
    });
    commands.spawn((
        CelestialBodyMarker,
        child_child_moon_node.dynamics.clone(),
        SpriteBundle {
            texture: asset_server.load("mike-petrucci-moon.png"),
            transform: Transform::from_scale(Vec3::splat(0.5)),
            ..default()
        },
        Ccd::enabled(),
        RigidBody::Fixed,
        AttractingBody,
        Collider::ball(sprite_radius),
        ColliderMassProperties::Mass(4e6),
        ActiveEvents::COLLISION_EVENTS,
        game_layer(),
    ));

    commands.insert_resource(OrbitHierarchy {
        roots: vec![parent_moon_node],
    });
}

pub fn update(
    time: Res<Time>,
    mut bodies: Query<(&mut Transform, &mut CircularOrbitChain), With<CelestialBodyMarker>>,
) {
    for (_, mut orbit) in bodies.iter_mut() {
        orbit.update(time.delta_seconds());
    }
    for (mut transform, orbit) in bodies.iter_mut() {
        transform.translation = orbit.pos(0.0).extend(0.0);
    }
}

pub fn cleanup(mut commands: Commands, bodies: Query<Entity, With<CelestialBodyMarker>>) {
    for entity in bodies.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
