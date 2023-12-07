use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier2d::dynamics::Velocity;
use colorgrad::CustomGradient;
use rand::{distributions::Uniform, Rng};

use crate::{
    camera::{game_layer, GameCameraMarker},
    thruster::Thruster,
};

use super::{Particle, ParticleKind};

pub fn spawn_thruster_particles(
    mut commands: Commands,
    time: Res<Time>,
    ships: Query<(&Transform, &Velocity, &Thruster)>,
    camera: Query<(&Transform, &OrthographicProjection), With<GameCameraMarker>>,
) {
    if let Ok((cam_transform, cam_proj)) = camera.get_single() {
        let cam_pos = cam_transform.translation.xy();
        let cam_area = cam_proj.area;
        let abs_cam_area = Rect {
            min: cam_area.min + cam_pos,
            max: cam_area.max + cam_pos,
        };

        let mut rng = rand::thread_rng();
        let max_angle_at_lowest_thrust = PI / 2.0;
        let max_particle_speed = 1000.0;
        for (transform, velocity, thruster) in ships.iter() {
            if abs_cam_area.contains(transform.translation.xy()) {
                let max_angle_at_current_thrust = (max_angle_at_lowest_thrust
                    * (1.0 - (thruster.current_thrust / thruster.max_thrust)).powf(0.6))
                .max(PI / 32.0);
                // debug!("max angle: {}", max_angle_at_current_thrust);
                let particle_angle_distribution =
                    Uniform::new(-max_angle_at_current_thrust, max_angle_at_current_thrust);
                let max_speed_at_current_thrust = (max_particle_speed
                    * (thruster.current_thrust / thruster.max_thrust).powf(2.0))
                .max(0.1);
                let particle_speed_distribution = Uniform::new(
                    max_speed_at_current_thrust / 2.0,
                    max_speed_at_current_thrust,
                );
                let n = (rng.gen::<f32>().abs()
                    * 40.0
                    * (thruster.current_thrust / thruster.max_thrust).powf(1.3))
                .floor() as u32;
                for _ in 0..n {
                    let theta = rng.sample(particle_angle_distribution);
                    let speed = rng.sample(particle_speed_distribution);
                    // debug!("theta, s: {}, {}", theta, speed);

                    let particle_vel = Vec2 {
                        x: theta.cos() * speed,
                        y: theta.sin() * speed,
                    };
                    let ship_particle_vel = particle_vel.rotate(transform.down().normalize().xy());
                    // let ship_particle_vel = particle_vel;
                    // debug!("Local particle velocity: {}", particle_vel);
                    // debug!("Ship particle velocity: {}", ship_particle_vel);
                    let pos = transform.translation.xy()
                        + velocity.linvel * time.delta_seconds()
                        + transform.down().xy().normalize() * 16.0;
                    let vel = velocity.linvel + ship_particle_vel;
                    let radius = rng.gen::<f32>().abs().powf(1.5) * 10.0 + 1.0;
                    commands.spawn((
                        TransformBundle::from_transform(Transform::from_translation(
                            pos.extend(-1.0),
                        )),
                        Particle {
                            lifetime: rng.gen::<f32>().abs().powf(2.0) * 0.25 + 0.1,
                            spawned_at: time.elapsed_seconds(),
                            // velocity: vel,
                            kind: ParticleKind::Combustion {
                                init_radius: 1.0,
                                end_radius: radius,
                                color: CustomGradient::new()
                                    .colors(&[
                                        colorgrad::Color::new(0.0, 0.5, 1.0, 1.0),
                                        colorgrad::Color::new(0.0, 0.5, 1.0, 1.0),
                                        colorgrad::Color::new(0.0, 0.25, 0.75, 0.75),
                                        colorgrad::Color::new(0.0, 0.25, 0.75, 0.75),
                                        colorgrad::Color::new(1.0, 1.0, 0.4, 0.5),
                                        colorgrad::Color::new(1.0, 0.0, 0.0, 0.5),
                                        colorgrad::Color::new(1.0, 1.0, 1.0, 0.25),
                                        colorgrad::Color::new(1.0, 1.0, 1.0, 0.0),
                                    ])
                                    .interpolation(colorgrad::Interpolation::Basis)
                                    .build()
                                    .unwrap(),
                            },
                        },
                        Velocity {
                            linvel: vel,
                            angvel: 0.0,
                        },
                        game_layer(),
                    ));
                }
            }
        }
    }
}
