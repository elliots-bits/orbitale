use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier2d::dynamics::Velocity;

use crate::{
    ai::{orientation_controller::OrientationController, OrientationControllerQueue},
    impulses_aggregator::AddExternalImpulse,
    lasers::{self, Laser, LaserAbility, LaserOrigin},
    particles::thrusters::spawn_rotation_thruster_cone,
    player::PlayerMarker,
    thruster::Thruster,
};

pub const ALIEN_SHIP_DRIVE_ENGINE_IMPULSE: f32 = 4.0;
pub const ALIEN_SHIP_ROTATION_IMPULSE: f32 = 8.0 * ALIEN_SHIP_DRIVE_ENGINE_IMPULSE;

pub const ALIEN_SHIP_LASER_COOLDOWN_S: f32 = 1.0;

const MAX_SHOOT_DISTANCE: f32 = 2000.0;
const MAX_SHOOT_THETA: f32 = PI / 8.0;
const MAX_DRIVE_THETA: f32 = PI / 8.0;

const ENABLE_SHOOTING: bool = false;
const AGGRO_RANGE: f32 = 2000.0;

/*
Draft outline of the ships' AI:

Every couple of seconds, the ship's trajectory will be simulated.

When the ship is far from the player, its behavior will depend on current the trajectory:
- Crashing into a celestial body -> execute a collision avoidance maneuver
- No crash planned: maneuver to intersect with player (accelerate towards player & minimize encounter distance, then start decelerating at the right time to match velocities when close to the player)

When the ship is near the player, it will ignore the planned trajectory and focus on attacking the player.
This allows the player to trick the enemy into crashing or being slingshot while it's attacking.

*/

#[derive(Component)]
pub struct AlienShipMarker;

pub fn update(
    mut commands: Commands,
    mut orientation_controller_queue: ResMut<OrientationControllerQueue>,
    time: Res<Time>,
    mut impulses: EventWriter<AddExternalImpulse>,
    player: Query<&Transform, With<PlayerMarker>>,
    mut query: Query<
        (
            Entity,
            &Transform,
            &Velocity,
            &OrientationController,
            &mut LaserAbility,
            &mut Thruster,
        ),
        With<AlienShipMarker>,
    >,
) {
    // The actual AI is going to be a bit tricky.
    // We'll at least have to implement a basic PID control loop.

    // For now, it is very dumb. It aims at the player and accelerates if it points in kinda in the player direction.
    if let Ok(player_t) = player.get_single() {
        for (entity, t, v, orientation_controller, mut laser_ability, mut thruster) in
            query.iter_mut()
        {
            let mut angular_impulse = 0.0;

            let local_forward = t.up().xy();
            let d = (player_t.translation - t.translation).xy();
            let orientation_to_player = local_forward.angle_between(d);
            if ENABLE_SHOOTING
                && d.length() < MAX_SHOOT_DISTANCE
                && orientation_to_player.abs() < MAX_SHOOT_THETA
                && laser_ability.ready(&time)
            {
                lasers::spawn(
                    &mut commands,
                    t.translation.xy()
                        + v.linvel * time.delta_seconds()
                        + t.up().xy().normalize() * 60.0,
                    local_forward.rotate(Vec2 { x: 1500.0, y: 0.0 }) + v.linvel,
                    Laser {
                        origin: LaserOrigin::Enemy,
                        damage: 10.0,
                        shot_at: time.elapsed_seconds(),
                    },
                );
                laser_ability.last_shot = Some(time.elapsed_seconds());
            }

            if orientation_to_player.abs() < MAX_DRIVE_THETA && d.length() > AGGRO_RANGE {
                thruster.throttle(time.delta_seconds());
            } else {
                thruster.release(time.delta_seconds());
            }
            let (cmd_torque, cmd_end_time) = orientation_controller.current_command;
            if time.elapsed_seconds() < cmd_end_time && cmd_torque.abs() > 0.01 {
                angular_impulse += cmd_torque;
                let particle_distance = 24.0;
                let xy = t.translation.xy();
                spawn_rotation_thruster_cone(
                    &mut commands,
                    &time,
                    xy + t.right().xy().normalize() * particle_distance * cmd_torque.signum(),
                    v.linvel,
                    t.down().xy().normalize() * cmd_torque.signum(),
                );
                // spawn_rotation_thruster_cone(
                //     &mut commands,
                //     &time,
                //     xy + t.left().xy().normalize() * particle_distance,
                //     v.linvel,
                //     t.up().xy().normalize() * cmd_torque.signum(),
                // );
            } else {
                orientation_controller_queue.0.push_back(entity);
            }
            impulses.send(AddExternalImpulse {
                entity,
                impulse: Vec2::ZERO,
                torque_impulse: angular_impulse,
            });
        }
    } else {
        error!("Can't process Alien Ships: player not found");
    }
}

pub fn cleanup(mut commands: Commands, ships: Query<Entity, With<AlienShipMarker>>) {
    for entity in ships.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
