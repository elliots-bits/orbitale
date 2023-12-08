use bevy::prelude::*;
use bevy_rapier2d::{
    dynamics::{Ccd, RigidBody, Velocity},
    geometry::{ActiveEvents, Collider, ColliderMassProperties, Sensor},
};
use bevy_vector_shapes::{painter::ShapePainter, shapes::RectPainter};

use crate::{camera::game_layer, despawn_queue::DespawnQueue, player::PlayerMarker};

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
    pub damage: f32,
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

pub fn draw(
    player: Query<&Velocity, With<PlayerMarker>>,
    query: Query<(&Transform, &Velocity, &Laser)>,
    mut painter: ShapePainter,
) {
    if let Ok(pv) = player.get_single() {
        for (transform, v, Laser { origin, .. }) in query.iter() {
            let dv = v.linvel - pv.linvel;
            let (color, size) = match origin {
                LaserOrigin::Enemy => (Color::hex("FF0000").unwrap(), Vec2::new(30.0, 2.0)),
                LaserOrigin::Player => (Color::hex("00FF80").unwrap(), Vec2::new(20.0, 3.0)),
            };
            painter.reset();
            painter.set_2d();
            painter.set_rotation(Quat::from_axis_angle(Vec3::Z, dv.y.atan2(dv.x)));
            painter.set_translation(transform.translation);
            painter.color = color;
            painter.rect(size);
        }
    }
}

pub fn spawn(commands: &mut Commands, position: Vec2, velocity: Vec2, props: Laser) {
    let mut transform = Transform::from_translation(Vec3 {
        x: position.x,
        y: position.y,
        z: 1.0,
    });
    commands.spawn((
        props,
        TransformBundle::from_transform(transform),
        RigidBody::KinematicVelocityBased,
        Ccd::enabled(),
        Collider::ball(2.0),
        ColliderMassProperties::Mass(1.0),
        Velocity::linear(velocity),
        ActiveEvents::COLLISION_EVENTS,
        Sensor,
        game_layer(),
    ));
}
