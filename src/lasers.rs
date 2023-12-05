use bevy::prelude::*;
use bevy_rapier2d::{
    dynamics::{Ccd, RigidBody, Velocity},
    geometry::{ActiveEvents, Collider},
};
use bevy_vector_shapes::{painter::ShapePainter, shapes::RectPainter};

use crate::despawn_queue::DespawnQueue;

pub const LASER_LIFETIME_S: f32 = 2.0;

#[derive(Component)]
pub struct LaserAbility {
    pub last_shot: Option<f32>,
    pub cooldown: f32,
}

impl LaserAbility {
    pub fn ready(&self, time: &Time) -> bool {
        self.last_shot.is_none()
            || time.elapsed_seconds() - self.last_shot.unwrap() >= self.cooldown
    }
}

#[derive(PartialEq, Eq)]
pub enum LaserOrigin {
    Player,
    Enemy,
}

#[derive(Component)]
pub struct Laser {
    pub origin: LaserOrigin,
    pub shot_at: f32,
}

pub fn update(
    mut despawn_queue: ResMut<DespawnQueue>,
    time: Res<Time>,
    query: Query<(Entity, &Laser)>,
) {
    for (entity, Laser { shot_at, .. }) in query.iter() {
        if time.elapsed_seconds() - shot_at > LASER_LIFETIME_S {
            despawn_queue.1.insert(entity);
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
        painter.rect(Vec2 { x: 20.0, y: 1.0 });
    }
}

pub fn spawn(
    time: &Time,
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
            shot_at: time.elapsed_seconds(),
        },
        TransformBundle::from_transform(transform),
        RigidBody::KinematicVelocityBased,
        Ccd::enabled(),
        Collider::ball(2.0),
        Velocity::linear(velocity),
        ActiveEvents::COLLISION_EVENTS,
    ));
}
