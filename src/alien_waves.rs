use bevy::prelude::*;
use bevy_rapier2d::{
    dynamics::{Ccd, Damping, RigidBody, Velocity},
    geometry::{Collider, ColliderMassProperties},
};
use rand::prelude::*;
use std::{
    f32::consts::PI,
    time::{Duration, Instant},
};

use crate::{
    alien_ship::{AlienShipMarker, ALIEN_SHIP_LASER_COOLDOWN_S},
    lasers::LaserAbility,
    player::PlayerMarker,
};

const WAVE_DURATION_S: f32 = 10.0;

#[derive(Resource)]
pub struct AlienWave {
    pub current_wave: u32,
    pub started_at: Option<Instant>,
}

pub fn setup(mut commands: Commands) {
    commands.insert_resource(AlienWave {
        current_wave: 1,
        started_at: None,
    });
}

pub fn update(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player: Query<&Transform, With<PlayerMarker>>,
    mut wave: ResMut<AlienWave>,
) {
    if let Ok(player_t) = player.get_single() {
        let mut rng = rand::thread_rng();
        let spawn_wave = wave.started_at.is_none()
            || wave.started_at.unwrap().elapsed().as_secs_f32() >= WAVE_DURATION_S;
        if spawn_wave {
            let n_to_spawn = 2_u32.pow(wave.current_wave);
            wave.current_wave += 1;
            wave.started_at = Some(Instant::now());
            // Spawn at random locations around player for now
            for _ in 0..n_to_spawn {
                let r = rng.gen::<f32>() * 500.0 + 500.0;
                let theta = rng.gen::<f32>() * 2.0 * PI;
                let pos = Vec3::new(
                    player_t.translation.x + theta.cos() * r,
                    player_t.translation.y + theta.sin() * r,
                    0.0,
                );
                commands.spawn((
                    AlienShipMarker,
                    LaserAbility {
                        last_shot: None,
                        cooldown: Duration::from_secs_f32(ALIEN_SHIP_LASER_COOLDOWN_S),
                    },
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
                        linear_damping: 0.05,
                        angular_damping: 0.5,
                    },
                    Velocity::default(),
                ));
            }
        }
    }
}
