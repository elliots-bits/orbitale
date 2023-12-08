use bevy::prelude::*;
use bevy_rapier2d::{
    dynamics::{Ccd, Damping, RigidBody, Velocity},
    geometry::{ActiveEvents, Collider, ColliderMassProperties},
};

use crate::{
    camera::game_layer,
    course_planner::ComputedTrajectory,
    gravity::AffectedByGravity,
    healthpoints::HealthPoints,
    impulses_aggregator::AddExternalImpulse,
    lasers::{self, Laser, LaserAbility, LaserOrigin},
    thruster::Thruster,
};

const PLAYER_MASS: f32 = 4.0;
const DRIVE_ENGINE_MAX_IMPULSE: f32 = 8.0 * PLAYER_MASS;
const DRIVE_ENGINE_INIT_IMPULSE: f32 = 3.0 * PLAYER_MASS;
const ROTATION_IMPULSE: f32 = 6.0 * DRIVE_ENGINE_MAX_IMPULSE;

const LASER_COOLDOWN_S: f32 = 0.02;

const STARTING_HP: f32 = 100.0;

#[derive(Component)]
pub struct PlayerMarker;

pub fn control(
    mut commands: Commands,
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
        if keys.pressed(KeyCode::Up) {
            thruster.throttle(time.delta_seconds());
        } else {
            thruster.release(time.delta_seconds());
        }
        if keys.pressed(KeyCode::Right) {
            angular_impulse -= ROTATION_IMPULSE;
        }
        if keys.pressed(KeyCode::Left) {
            angular_impulse += ROTATION_IMPULSE;
        }
        let local_forward = transform.up().xy();
        if keys.pressed(KeyCode::Space) && laser_ability.ready(&time) {
            let laser_angle = local_forward.y.atan2(local_forward.x);
            lasers::spawn(
                &mut commands,
                transform.translation.xy()
                    + velocity.linvel * time.delta_seconds()
                    + transform.up().xy().normalize() * 40.0,
                Vec2 { x: 4000.0, y: 0.0 }.rotate(local_forward) + velocity.linvel,
                laser_angle,
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
            torque_impulse: angular_impulse,
        });
    }
}

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    debug!("Player setup");
    commands.spawn((
        PlayerMarker,
        HealthPoints {
            max: STARTING_HP,
            current: STARTING_HP,
        },
        Thruster {
            max_thrust: DRIVE_ENGINE_MAX_IMPULSE,
            current_thrust: 0.0,
            rampup_rate: 1.8,
            shutoff_rate: 7.0,
            ignition_thrust: DRIVE_ENGINE_INIT_IMPULSE,
        },
        LaserAbility {
            last_shot: None,
            cooldown: LASER_COOLDOWN_S,
        },
        ComputedTrajectory::default(),
        SpriteBundle {
            texture: asset_server.load("spaceship_dev1.png"),
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
        Velocity::default(),
        ActiveEvents::COLLISION_EVENTS,
        AffectedByGravity::default(),
        game_layer(),
    ));
}

pub fn cleanup(mut commands: Commands, query: Query<Entity, With<PlayerMarker>>) {
    debug!("Cleanup player");
    commands.entity(query.single()).despawn_recursive();
}
