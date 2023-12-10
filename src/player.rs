use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier2d::{
    dynamics::{Ccd, Damping, RigidBody, Velocity},
    geometry::{ActiveEvents, Collider, ColliderMassProperties},
};

use crate::{
    camera::game_layer,
    celestial_body::{CircularOrbitChain, StarterPlanetMarker},
    course_planner::ComputedTrajectory,
    gravity::AffectedByGravity,
    healthpoints::HealthPoints,
    impulses_aggregator::AddExternalImpulse,
    lasers::{self, Laser, LaserAbility, LaserOrigin},
    particles::thrusters::spawn_rotation_thruster_cone,
    thruster::Thruster,
    ui::GameSettings,
    GLOBAL_IMPULSE_DURATION_MULT,
};

const PLAYER_MASS: f32 = 4.0;
const DRIVE_ENGINE_MAX_IMPULSE: f32 = 8.0 * PLAYER_MASS;
const DRIVE_ENGINE_INIT_IMPULSE: f32 = 3.0 * PLAYER_MASS;
const ROTATION_IMPULSE: f32 = 16.0 * DRIVE_ENGINE_MAX_IMPULSE;

const LASER_COOLDOWN_S: f32 = 0.02;

const STARTING_HP: f32 = 100.0;

#[derive(Component)]
pub struct PlayerMarker;

pub fn control(
    mut commands: Commands,
    settings: Res<GameSettings>,
    time: Res<Time>,
    mut impulses: EventWriter<AddExternalImpulse>,
    mut player: Query<
        (
            Entity,
            &mut LaserAbility,
            &mut Thruster,
            &Transform,
            &Velocity,
        ),
        With<PlayerMarker>,
    >,
    keys: Res<Input<KeyCode>>,
) {
    if let Ok((entity, mut laser_ability, mut thruster, transform, velocity)) =
        player.get_single_mut()
    {
        let mut angular_impulse = 0.0;
        let xy = transform.translation.xy();
        let particle_distance = 24.0;
        if keys.pressed(KeyCode::Up) {
            thruster.throttle(time.delta_seconds());
        } else {
            thruster.release(time.delta_seconds());
        }

        if keys.pressed(KeyCode::Right) {
            angular_impulse -= ROTATION_IMPULSE;
            spawn_rotation_thruster_cone(
                &mut commands,
                settings.entities_quantity,
                &time,
                xy + transform.right().xy().normalize() * particle_distance,
                velocity.linvel,
                transform.up().xy().normalize(),
            );
            spawn_rotation_thruster_cone(
                &mut commands,
                settings.entities_quantity,
                &time,
                xy + transform.left().xy().normalize() * particle_distance,
                velocity.linvel,
                transform.down().xy().normalize(),
            );
        } else if keys.pressed(KeyCode::Left) {
            angular_impulse += ROTATION_IMPULSE;
            spawn_rotation_thruster_cone(
                &mut commands,
                settings.entities_quantity,
                &time,
                xy + transform.right().xy().normalize() * particle_distance,
                velocity.linvel,
                transform.down().xy().normalize(),
            );
            spawn_rotation_thruster_cone(
                &mut commands,
                settings.entities_quantity,
                &time,
                xy + transform.left().xy().normalize() * particle_distance,
                velocity.linvel,
                transform.up().xy().normalize(),
            );
        }
        let local_forward = transform.up().xy();
        if keys.pressed(KeyCode::Space) && laser_ability.ready(&time) {
            lasers::spawn(
                &mut commands,
                transform.translation.xy()
                    + velocity.linvel * time.delta_seconds()
                    + transform.up().xy().normalize() * 40.0,
                Vec2 { x: 3000.0, y: 0.0 }.rotate(local_forward) + velocity.linvel,
                Laser {
                    origin: LaserOrigin::Player,
                    damage: 100.0,
                    shot_at: time.elapsed_seconds(),
                },
            );
            laser_ability.last_shot = Some(time.elapsed_seconds());
        }

        impulses.send(AddExternalImpulse {
            entity,
            impulse: Vec2::ZERO,
            torque_impulse: angular_impulse * time.delta_seconds() * GLOBAL_IMPULSE_DURATION_MULT,
        });
    }
}

pub fn setup(
    mut commands: Commands,
    player: Query<&PlayerMarker>,
    asset_server: Res<AssetServer>,
    starter_planet: Query<(&StarterPlanetMarker, &CircularOrbitChain)>,
) {
    if player.is_empty() {
        debug!("Player setup");
        if let Ok((player_orbit, planet)) = starter_planet.get_single() {
            let planet_pos = planet.pos(0.0);
            let player_pos = Vec2 {
                x: player_orbit.theta.cos() * player_orbit.orbit_radius,
                y: player_orbit.theta.sin() * player_orbit.orbit_radius,
            } + planet_pos;
            commands.spawn((
                PlayerMarker,
                HealthPoints {
                    max: STARTING_HP,
                    current: STARTING_HP,
                },
                Thruster {
                    max_thrust: DRIVE_ENGINE_MAX_IMPULSE,
                    current_thrust: 0.0,
                    rampup_rate: 10.0,
                    shutoff_rate: DRIVE_ENGINE_MAX_IMPULSE * 2.0,
                    ignition_thrust: DRIVE_ENGINE_INIT_IMPULSE,
                },
                LaserAbility {
                    last_shot: None,
                    cooldown: LASER_COOLDOWN_S,
                },
                ComputedTrajectory::default(),
                SpriteBundle {
                    texture: asset_server.load("player_ship.png"),
                    transform: Transform::from_translation(player_pos.extend(0.0))
                        .with_scale(Vec3::splat(1.5)),
                    ..default()
                },
                Ccd::enabled(),
                RigidBody::Dynamic,
                Collider::ball(32.0),
                ColliderMassProperties::Mass(PLAYER_MASS),
                Damping {
                    linear_damping: 0.0,
                    angular_damping: 2.0,
                },
                Velocity {
                    linvel: player_orbit.velocity,
                    angvel: PI,
                },
                ActiveEvents::COLLISION_EVENTS,
                AffectedByGravity::default(),
                game_layer(),
            ));
        }
    }
}

pub fn cleanup(mut commands: Commands, query: Query<Entity, With<PlayerMarker>>) {
    debug!("Cleanup player");
    commands.entity(query.single()).despawn_recursive();
}
