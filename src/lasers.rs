use std::{
    f32::consts::PI,
    time::{Duration, Instant},
};

use bevy::prelude::*;
use bevy_rapier2d::{
    dynamics::{Ccd, RigidBody, Velocity},
    geometry::{Collider, ColliderMassProperties},
};
use bevy_vector_shapes::{painter::ShapePainter, shapes::RectPainter};

pub const LASER_LIFETIME_S: f32 = 5.0;

pub enum LaserOrigin {
    Player,
    Enemy,
}

#[derive(Component)]
pub struct Laser {
    pub origin: LaserOrigin,
    pub shot_at: Instant,
}

pub fn update(mut commands: Commands, query: Query<(Entity, &Laser)>) {
    for (entity, Laser { shot_at, .. }) in query.iter() {
        if shot_at.elapsed().as_secs_f32() > LASER_LIFETIME_S {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn draw(query: Query<(&Transform, &Laser)>, mut painter: ShapePainter) {
    for (transform, Laser { origin, .. }) in query.iter() {
        let color = match origin {
            LaserOrigin::Enemy => Color::hex("FF0000").unwrap(),
            LaserOrigin::Player => Color::hex("00FF00").unwrap(),
        };
        painter.reset();
        painter.set_2d();
        painter.set_rotation(transform.rotation);
        painter.set_translation(transform.translation);
        painter.color = color;
        painter.rect(Vec2 { x: 1.0, y: 10.0 });
    }
}

pub fn spawn(
    commands: &mut Commands,
    position: Vec2,
    velocity: Vec2,
    angle: f32,
    origin: LaserOrigin,
) {
    let mut transform = Transform::from_translation(Vec3 {
        x: position.x,
        y: position.y,
        z: 1.0,
    });
    transform.rotate_axis(Vec3::Z, angle);
    commands.spawn((
        Laser {
            origin,
            shot_at: Instant::now(),
        },
        TransformBundle::from_transform(transform),
        RigidBody::Dynamic,
        Ccd::enabled(),
        ColliderMassProperties::Mass(0.001),
        Collider::cuboid(1.0, 10.0),
        Velocity::linear(velocity),
    ));
}
