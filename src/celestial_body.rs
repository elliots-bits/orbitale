use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier2d::{
    dynamics::{Ccd, RigidBody},
    geometry::{ActiveEvents, Collider, ColliderMassProperties, Restitution},
};

use crate::{camera::game_layer, gravity::AttractingBody};

const SYSTEM_DISTANCE_SCALE: f32 = 10000.0;

#[derive(Component)]
pub struct CelestialBodyMarker;

#[derive(Component)]
pub struct StarterPlanetMarker {
    pub orbit_radius: f32,
    pub theta: f32,
    pub velocity: Vec2,
}

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
    let sprite_radius = 500.0;

    let mut parent_saturn = OrbitHierarchyNode::start(Vec2 {
        x: -10000.0,
        y: -10000.0,
    });
    commands.spawn(gen_body_bundle(
        &parent_saturn,
        5.0,
        asset_server.load("planet_big.webp"),
        sprite_radius,
        5e7,
    ));

    let mut parent_massive_pink_hole = OrbitHierarchyNode::start(Vec2 {
        x: 0.0,
        y: 100000.0,
    });
    commands.spawn(gen_body_bundle(
        &parent_massive_pink_hole,
        3.0,
        asset_server.load("planet_pink_shiny_rock.webp"),
        sprite_radius,
        7e7,
    ));

    let volcanic = parent_massive_pink_hole.with_child(CircularOrbitDef {
        theta: -PI / 2.0,
        radius: 50000.0,
        freq: 1.0 / 30.0,
    });
    commands.spawn(gen_body_bundle(
        &volcanic,
        5.0,
        asset_server.load("planet_big_1.webp"),
        sprite_radius,
        5e7,
    ));

    let parent_pink_rock = OrbitHierarchyNode::start(Vec2 {
        x: 70000.0,
        y: -70000.0,
    });
    commands.spawn(gen_body_bundle(
        &parent_pink_rock,
        5.0,
        asset_server.load("planet_pink_shiny_rock.webp"),
        sprite_radius,
        5e7,
    ));

    {
        let child_moon_node = parent_saturn.with_child(CircularOrbitDef {
            theta: 0.0,
            radius: 4.0 * SYSTEM_DISTANCE_SCALE,
            freq: 1.0 / 300.0,
        });
        commands.spawn(gen_body_bundle(
            child_moon_node,
            1.5,
            asset_server.load("planet_small_0.webp"),
            sprite_radius,
            1e7,
        ));

        let child_child_moon_node = child_moon_node.with_child(CircularOrbitDef {
            theta: 0.0,
            radius: 1.0 * SYSTEM_DISTANCE_SCALE,
            freq: 1.0 / 40.0,
        });
        commands.spawn((
            gen_body_bundle(
                child_child_moon_node,
                0.5,
                asset_server.load("planet_small_3.webp"),
                sprite_radius,
                4e6,
            ),
            StarterPlanetMarker {
                orbit_radius: 1500.0,
                theta: PI / 2.0,
                velocity: -Vec2::X * 1000.0,
            },
        ));
    }

    let child_binary_center = parent_saturn.with_child(CircularOrbitDef {
        theta: PI,
        radius: 8.0 * SYSTEM_DISTANCE_SCALE,
        freq: 1.0 / 600.0,
    });
    {
        let binary_a = child_binary_center.with_child(CircularOrbitDef {
            theta: 0.0,
            radius: 0.08 * SYSTEM_DISTANCE_SCALE,
            freq: 1.0,
        });
        commands.spawn(gen_body_bundle(
            binary_a,
            0.5,
            asset_server.load("planet_blue_shiny_rock.webp"),
            sprite_radius,
            3e6,
        ));
    }
    {
        let binary_b = child_binary_center.with_child(CircularOrbitDef {
            theta: PI,
            radius: 0.08 * SYSTEM_DISTANCE_SCALE,
            freq: 1.0,
        });
        commands.spawn(gen_body_bundle(
            binary_b,
            0.5,
            asset_server.load("planet_small_1.webp"),
            sprite_radius,
            3e6,
        ));
    }

    commands.insert_resource(OrbitHierarchy {
        roots: vec![parent_saturn, parent_massive_pink_hole, parent_pink_rock],
    });
}

fn gen_body_bundle(
    node: &OrbitHierarchyNode,
    scale: f32,
    texture: Handle<Image>,
    sprite_radius: f32,
    mass: f32,
) -> impl Bundle {
    (
        CelestialBodyMarker,
        node.dynamics.clone(),
        SpriteBundle {
            texture: texture.clone(),
            transform: Transform::from_scale(Vec3::splat(scale)),
            ..default()
        },
        Ccd::enabled(),
        RigidBody::Fixed,
        AttractingBody,
        Collider::ball(sprite_radius),
        ColliderMassProperties::Mass(mass),
        ActiveEvents::COLLISION_EVENTS,
        Restitution::coefficient(1.0),
        game_layer(),
    )
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
