use bevy::prelude::*;
use bevy_rapier2d::{
    dynamics::{Ccd, Damping, RigidBody, Velocity},
    geometry::{ActiveEvents, Collider, ColliderMassProperties},
};
use rand::prelude::*;
use std::f32::consts::PI;

use crate::{
    ai::{
        orientation_controller::OrientationController, position_controller::PositionController,
        AIControllerQueues, ShipAi,
    },
    alien_ship::{
        AlienShipMarker, ALIEN_SHIP_DRIVE_ENGINE_IMPULSE, ALIEN_SHIP_LASER_COOLDOWN_S,
        ALIEN_SHIP_MASS, ALIEN_SHIP_ROTATION_IMPULSE,
    },
    camera::game_layer,
    course_planner::ComputedTrajectory,
    gravity::AffectedByGravity,
    healthpoints::HealthPoints,
    lasers::LaserAbility,
    player::PlayerMarker,
    thruster::Thruster,
    ui::{EntitiesQuantity, GameSettings},
};
use rand::distributions::Uniform;

const ENABLE_ENEMIES: bool = true;
const WAVE_DURATION_S: f32 = 30.0;

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

fn enemies_per_wave_count(wave: &ResMut<AlienWave>, settings: &Res<GameSettings>) -> u32 {
    wave.current_wave
        * match settings.entities_quantity {
            EntitiesQuantity::Some => 5,
            EntitiesQuantity::ALot => 10,
            EntitiesQuantity::TooMuch => 20,
        }
}

fn should_start_next_wave(
    wave: &ResMut<AlienWave>,
    time: &Res<Time>,
    settings: &Res<GameSettings>,
    enemies_query: Query<Entity, With<AlienShipMarker>>,
) -> bool {
    ENABLE_ENEMIES
        && (wave.started_at.is_none()
            || time.elapsed_seconds() - wave.started_at.unwrap() >= WAVE_DURATION_S
            || enemies_query.iter().count()
                < (enemies_per_wave_count(wave, settings) / 10) as usize)
}

pub fn update(
    mut commands: Commands,
    mut controller_queue: ResMut<AIControllerQueues>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    player: Query<(&Transform, &Velocity), With<PlayerMarker>>,
    mut wave: ResMut<AlienWave>,
    settings: Res<GameSettings>,
    enemies_query: Query<Entity, With<AlienShipMarker>>,
) {
    if let Ok((player_transform, player_velocity)) = player.get_single() {
        let mut rng = rand::thread_rng();

        if should_start_next_wave(&wave, &time, &settings, enemies_query) {
            let angle_side = Uniform::new(0.0, PI * 2.0);
            let radius_side = Uniform::new(50.0, 2000.0);
            let n_to_spawn = enemies_per_wave_count(&wave, &settings);
            debug!("Spawning {} alien ships", n_to_spawn);

            let r = 5000.;
            let theta = rng.sample(angle_side);

            let wave_center = Vec3::new(
                player_transform.translation.x + theta.cos() * r,
                player_transform.translation.y + theta.sin() * r,
                0.0,
            );

            for _ in 0..n_to_spawn {
                let r = rng.sample(radius_side);
                let theta = rng.sample(angle_side);
                let pos = Vec3::new(
                    wave_center.x + theta.cos() * r,
                    wave_center.y + theta.sin() * r,
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
                    PositionController::new(ALIEN_SHIP_DRIVE_ENGINE_IMPULSE * 0.75), // smaller than max thrust to leave some error margin on slowdown maneuvers
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
                    ColliderMassProperties::Mass(ALIEN_SHIP_MASS),
                    Damping {
                        linear_damping: 0.0,
                        angular_damping: 0.5,
                    },
                    Velocity {
                        linvel: player_velocity.linvel,
                        ..default()
                    },
                ));
                controller_queue.queue_spawned(cmd.id());
            }
            wave.current_wave += 1;
            wave.started_at = Some(time.elapsed_seconds());
        }
    }
}
