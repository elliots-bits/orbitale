use bevy::{prelude::*, render::view::RenderLayers};
use bevy_rapier2d::{
    dynamics::{Ccd, RigidBody, Velocity},
    geometry::{ActiveEvents, Collider, ColliderMassProperties, Sensor},
};
use bevy_vector_shapes::{painter::ShapePainter, shapes::RectPainter};

use crate::{
    camera::{game_layer, GameCameraMarker, UI_LAYER},
    despawn_queue::DespawnQueue,
    gravity::AffectedByGravity,
    player::PlayerMarker,
};

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
    time: Res<Time>,
    player: Query<(&Transform, &Velocity), With<PlayerMarker>>,
    camera: Query<(&Transform, &OrthographicProjection), With<GameCameraMarker>>,
    query: Query<(&Transform, &Velocity, &Laser)>,
    mut painter: ShapePainter,
) {
    if let Ok((pt, pv)) = player.get_single() {
        if let Ok((_cam_transform, cam_proj)) = camera.get_single() {
            for (transform, v, Laser { origin, .. }) in query.iter() {
                let dp = (transform.translation
                    - pt.translation
                    - pv.linvel.extend(0.0) * time.delta_seconds())
                    / cam_proj.scale;
                let dv = v.linvel - pv.linvel;
                let (color, size) = match origin {
                    LaserOrigin::Enemy => (Color::hex("FF0000").unwrap(), Vec2::new(40.0, 5.0)),
                    LaserOrigin::Player => (Color::hex("00FF80").unwrap(), Vec2::new(30.0, 5.0)),
                };
                painter.reset();
                painter.set_2d();
                painter.render_layers = Some(RenderLayers::layer(UI_LAYER));
                painter.set_rotation(Quat::from_axis_angle(Vec3::Z, dv.y.atan2(dv.x)));
                painter.set_translation(dp);
                painter.color = color;
                painter.rect(size / cam_proj.scale);
            }
        }
    }
}

pub fn spawn(commands: &mut Commands, position: Vec2, velocity: Vec2, props: Laser) {
    let transform = Transform::from_translation(Vec3 {
        x: position.x,
        y: position.y,
        z: 1.0,
    });
    commands.spawn((
        props,
        TransformBundle::from_transform(transform),
        RigidBody::KinematicVelocityBased,
        Ccd::enabled(),
        Collider::ball(4.0),
        ColliderMassProperties::Mass(1.0),
        Velocity::linear(velocity),
        ActiveEvents::COLLISION_EVENTS,
        Sensor,
        AffectedByGravity::default(),
        game_layer(),
    ));
}
