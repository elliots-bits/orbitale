pub mod thrusters;

use bevy::{prelude::*, render::view::RenderLayers};
use bevy_rapier2d::dynamics::Velocity;
use bevy_vector_shapes::{
    painter::ShapePainter,
    shapes::{DiscPainter, RectPainter},
};

use crate::{camera::GAME_LAYER, despawn_queue::DespawnQueue};

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
    Spark {
        init_size: Vec2,
        end_size: Vec2,
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

pub fn draw(mut painter: ShapePainter, time: Res<Time>, particles: Query<(&Particle, &Transform)>) {
    for (particle, transform) in particles.iter() {
        painter.reset();
        painter.set_2d();
        painter.render_layers = Some(RenderLayers::layer(GAME_LAYER));
        painter.set_rotation(transform.rotation);
        painter.set_translation(transform.translation);
        painter.hollow = false;
        let lifetime_frac =
            ((time.elapsed_seconds() - particle.spawned_at) / particle.lifetime).clamp(0.0, 1.0);
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
                painter.circle(lerp(*init_radius, *end_radius, lifetime_frac.powf(2.0)));
            }
            ParticleKind::Spark {
                init_size,
                end_size,
                color,
            } => {
                let color = color.at(lifetime_frac as f64);
                painter.color = Color::rgba(
                    color.r as f32,
                    color.g as f32,
                    color.b as f32,
                    color.a as f32,
                );
                painter.rect(init_size.lerp(*end_size, lifetime_frac));
            }
        }
    }
}

fn lerp(a: f32, b: f32, x: f32) -> f32 {
    a + x * (b - a)
}
