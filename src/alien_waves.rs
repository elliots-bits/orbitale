use bevy::prelude::*;
use bevy_rapier2d::{
    dynamics::{Ccd, Damping, RigidBody, Velocity},
    geometry::{ActiveEvents, Collider, ColliderMassProperties},
};
use rand::prelude::*;
use std::f32::consts::PI;

use crate::{
    alien_ship::{AlienShipMarker, ALIEN_SHIP_LASER_COOLDOWN_S},
    camera::game_layer,
    course_planner::ComputedTrajectory,
    gravity::AffectedByGravity,
    healthpoints::HealthPoints,
    lasers::LaserAbility,
    player::PlayerMarker,
    thruster::Thruster,
};
use rand::distributions::Uniform;

const ENABLE_ENEMIES: bool = true;
const WAVE_DURATION_S: f32 = 10.0;

#[derive(Resource)]
pub struct AlienWave {
    pub current_wave: u32,
    pub started_at: Option<f32>,
}

pub fn setup(mut commands: Commands) {
    commands.insert_resource(AlienWave {
        current_wave: 1,
        started_at: None,
    });
}

pub fn update(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    player: Query<&Transform, With<PlayerMarker>>,
    mut wave: ResMut<AlienWave>,
) {
    if let Ok(player_t) = player.get_single() {
        let mut rng = rand::thread_rng();
        let spawn_wave = ENABLE_ENEMIES
            && (wave.started_at.is_none()
                || time.elapsed_seconds() - wave.started_at.unwrap() >= WAVE_DURATION_S);
        if spawn_wave {
            let angle_side = Uniform::new(0.0, PI * 2.0);
            let radius_side = Uniform::new(2000.0, 10000.0);
            let n_to_spawn = wave.current_wave * 20;
            debug!("Spawning {} alien ships", n_to_spawn);
            // Spawn at random locations around player for now
            for _ in 0..n_to_spawn {
                let r = rng.sample(radius_side);
                let theta = rng.sample(angle_side);
                let pos = Vec3::new(
                    player_t.translation.x + theta.cos() * r,
                    player_t.translation.y + theta.sin() * r,
                    0.0,
                );
                commands.spawn((
                    AlienShipMarker,
                    HealthPoints {
                        max: 10.0,
                        current: 10.0,
                    },
                    Thruster {
                        max_thrust: 4.0,
                        current_thrust: 0.0,
                        rampup_rate: 2.0,
                        shutoff_rate: 7.0,
                        ignition_thrust: 2.0,
                    },
                    LaserAbility {
                        last_shot: None,
                        cooldown: ALIEN_SHIP_LASER_COOLDOWN_S,
                    },
                    ComputedTrajectory::default(),
                    SpriteBundle {
                        texture: asset_server.load("spaceship_dev1.png"),
                        transform: Transform::from_translation(pos),
                        ..default()
                    },
                    Ccd::enabled(),
                    RigidBody::Dynamic,
                    Collider::ball(32.0),
                    ColliderMassProperties::Mass(1.0),
                    Damping {
                        linear_damping: 0.0,
                        angular_damping: 0.5,
                    },
                    Velocity::default(),
                    ActiveEvents::COLLISION_EVENTS,
                    AffectedByGravity,
                    game_layer(),
                ));
            }
            wave.current_wave += 1;
            wave.started_at = Some(time.elapsed_seconds());
        }
    }
}
