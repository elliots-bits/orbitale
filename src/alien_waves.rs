use bevy::prelude::*;
use bevy_rapier2d::{
    dynamics::{Ccd, Damping, RigidBody, Velocity},
    geometry::{ActiveEvents, Collider, ColliderMassProperties},
};
use rand::prelude::*;
use std::f32::consts::PI;

use crate::{
    ai::{
        orientation_controller::{OrientationController, OrientationControllerQueue},
        ShipAi,
    },
    alien_ship::{
        AlienShipMarker, ALIEN_SHIP_DRIVE_ENGINE_IMPULSE, ALIEN_SHIP_LASER_COOLDOWN_S,
        ALIEN_SHIP_ROTATION_IMPULSE,
    },
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
    mut controller_queue: ResMut<OrientationControllerQueue>,
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
            let n_to_spawn = wave.current_wave * 10;
            // let n_to_spawn = 1;
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
                let mut cmd: bevy::ecs::system::EntityCommands<'_, '_, '_> = commands.spawn((
                    AlienShipMarker,
                    HealthPoints {
                        max: 10.0,
                        current: 10.0,
                    },
                    Thruster {
                        max_thrust: ALIEN_SHIP_DRIVE_ENGINE_IMPULSE,
                        current_thrust: 0.0,
                        rampup_rate: 2.0,
                        shutoff_rate: 7.0,
                        ignition_thrust: 2.0,
                    },
                    ShipAi::default(),
                    OrientationController::new(ALIEN_SHIP_ROTATION_IMPULSE),
                    LaserAbility {
                        last_shot: None,
                        cooldown: ALIEN_SHIP_LASER_COOLDOWN_S,
                    },
                    ComputedTrajectory::default(),
                    SpriteBundle {
                        texture: asset_server.load("enemy_ship.png"),
                        transform: Transform::from_translation(pos),
                        ..default()
                    },
                    ActiveEvents::COLLISION_EVENTS,
                    AffectedByGravity::default(),
                    game_layer(),
                ));
                cmd.insert((
                    Ccd::enabled(),
                    RigidBody::Dynamic,
                    Collider::ball(32.0),
                    ColliderMassProperties::Mass(1.0),
                    Damping {
                        linear_damping: 0.0,
                        angular_damping: 0.5,
                    },
                    Velocity::default(),
                ));
                controller_queue.0.push_back(cmd.id());
            }
            wave.current_wave += 1;
            wave.started_at = Some(time.elapsed_seconds());
        }
    }
}
