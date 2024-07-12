pub mod thrusters;

use bevy::{prelude::*, render::view::RenderLayers};
use bevy_rapier2d::dynamics::Velocity;
use bevy_vector_shapes::{
    painter::ShapePainter,
    shapes::{DiscPainter, RectPainter},
};

use crate::{
    camera::{GameCameraMarker, UI_LAYER},
    despawn_queue::DespawnQueue,
    player::PlayerMarker,
};

#[derive(Component)]
pub struct Particle {
    pub lifetime: f32,
    pub spawned_at: f32,
    pub kind: ParticleKind,
}

pub enum ParticleKind {
    Combustion {
        init_radius: f32,
        end_radius: f32,
        color: colorgrad::Gradient,
    },
}

pub fn update(
    mut despawn_queue: ResMut<DespawnQueue>,
    time: Res<Time>,
    mut particles: Query<(Entity, &mut Transform, &Velocity, &Particle)>,
) {
    for (
        entity,
        mut transform,
        velocity,
        &Particle {
            lifetime,
            spawned_at,
            ..
        },
    ) in particles.iter_mut()
    {
        if time.elapsed_seconds() - spawned_at >= lifetime {
            despawn_queue.0.insert(entity);
        } else {
            transform.translation += velocity.linvel.extend(0.0) * time.delta_seconds();
        }
    }
}

pub fn draw(
    camera: Query<&OrthographicProjection, With<GameCameraMarker>>,
    mut painter: ShapePainter,
    player: Query<(&Transform, &Velocity), With<PlayerMarker>>,
    time: Res<Time>,
    particles: Query<(&Particle, &Transform)>,
) {
    // We're are rendering on the UI layer with horrible tricks
    // because there is a bug somewhere in bevy_vector_shapes that affects WebGL rendering
    // and causes flickers on the radar IF we render this on the game layer.
    // does not seem like a z-fighting or concurrency issue
    // haven't figured out the root cause yet.
    if let Ok(cam_proj) = camera.get_single() {
        if let Ok((pt, pv)) = player.get_single() {
            for (particle, transform) in particles.iter() {
                let pos = transform.translation
                    - pt.translation
                    - (pv.linvel * time.delta_seconds()).extend(0.0);
                painter.reset();
                painter.set_2d();

                painter.render_layers = Some(RenderLayers::layer(UI_LAYER));
                painter.set_rotation(transform.rotation);
                painter.set_translation(pos / cam_proj.scale);
                painter.hollow = false;
                let lifetime_frac = ((time.elapsed_seconds() - particle.spawned_at)
                    / particle.lifetime)
                    .clamp(0.0, 1.0);
                match &particle.kind {
                    ParticleKind::Combustion {
                        init_radius,
                        end_radius,
                        color,
                    } => {
                        let color = color.at(lifetime_frac as f64);
                        painter.color = Color::rgba(
                            color.r as f32,
                            color.g as f32,
                            color.b as f32,
                            color.a as f32,
                        );
                        painter.circle(
                            lerp(*init_radius, *end_radius, lifetime_frac.powf(2.0))
                                / cam_proj.scale,
                        );
                    }
                }
            }
        }
    }
}

fn lerp(a: f32, b: f32, x: f32) -> f32 {
    a + x * (b - a)
}
