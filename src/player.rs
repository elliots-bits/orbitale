use bevy::prelude::*;
use bevy_rapier2d::{
    dynamics::{Ccd, Damping, RigidBody, Velocity},
    geometry::{ActiveEvents, Collider, ColliderMassProperties},
};

use crate::{
    camera::game_layer,
    impulses_aggregator::AddExternalImpulse,
    lasers::{self, Laser, LaserAbility, LaserOrigin},
    thruster::Thruster,
};

const DRIVE_ENGINE_IMPULSE: f32 = 6.0;
const LASER_KNOCKBACK_IMPULSE: f32 = 3.0;
const ROTATION_MUL: f32 = 8.0;

const LASER_COOLDOWN_S: f32 = 0.025;

const STARTING_HP: f32 = 100.0;

#[derive(Component)]
pub struct PlayerMarker;

#[derive(Component)]
pub struct PlayerHP {
    pub max: f32,
    pub current: f32,
}

impl PlayerHP {
    pub fn decrease(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }
}

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
        let mut linear_impulse = Vec2::ZERO;
        let mut angular_impulse = 0.0;
        if keys.pressed(KeyCode::Up) {
            thruster.rampup(time.delta_seconds());
        } else {
            thruster.shutoff(time.delta_seconds());
        }
        if keys.pressed(KeyCode::Right) {
            angular_impulse -= DRIVE_ENGINE_IMPULSE * ROTATION_MUL;
        }
        if keys.pressed(KeyCode::Left) {
            angular_impulse += DRIVE_ENGINE_IMPULSE * ROTATION_MUL;
        }
        let local_forward = transform.up().xy();
        if keys.pressed(KeyCode::Space) && laser_ability.ready(&time) {
            let laser_angle = local_forward.y.atan2(local_forward.x);
            lasers::spawn(
                &mut commands,
                transform.translation.xy() + transform.up().xy().normalize() * 40.0,
                Vec2 { x: 4000.0, y: 0.0 }.rotate(local_forward) + velocity.linvel,
                laser_angle,
                Laser {
                    origin: LaserOrigin::Player,
                    damage: 100.0,
                    shot_at: time.elapsed_seconds(),
                },
            );
            laser_ability.last_shot = Some(time.elapsed_seconds());
            linear_impulse.x -= LASER_KNOCKBACK_IMPULSE;
        }

        impulses.send(AddExternalImpulse {
            entity,
            impulse: local_forward.rotate(linear_impulse),
            torque_impulse: angular_impulse,
        });
    }
}

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        PlayerMarker,
        PlayerHP {
            max: STARTING_HP,
            current: STARTING_HP,
        },
        Thruster {
            max_thrust: 8.0,
            current_thrust: 0.0,
            rampup_rate: 1.5,
            shutoff_rate: 5.0,
            ignition_thrust: 3.0,
        },
        LaserAbility {
            last_shot: None,
            cooldown: LASER_COOLDOWN_S,
        },
        SpriteBundle {
            texture: asset_server.load("spaceship_dev1.png"),
            ..default()
        },
        Ccd::enabled(),
        RigidBody::Dynamic,
        Collider::ball(32.0),
        ColliderMassProperties::Mass(1.0),
        Damping {
            linear_damping: 0.0,
            angular_damping: 2.0,
        },
        Velocity::default(),
        ActiveEvents::COLLISION_EVENTS,
        game_layer(),
    ));
}

pub fn cleanup(mut commands: Commands, query: Query<Entity, With<PlayerMarker>>) {
    commands.entity(query.single()).despawn_recursive();
}
