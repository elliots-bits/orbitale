use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier2d::dynamics::Velocity;

use crate::{
    ai::{
        orientation_controller::OrientationController, position_controller::PositionController,
        AIControllerQueues, AGGRO_RANGE,
    },
    camera::GameCameraMarker,
    impulses_aggregator::AddExternalImpulse,
    lasers::{self, Laser, LaserAbility, LaserOrigin},
    particles::thrusters::spawn_rotation_thruster_cone,
    player::PlayerMarker,
    thruster::Thruster,
    ui::GameSettings,
    GLOBAL_IMPULSE_DURATION_MULT,
};

pub const ALIEN_SHIP_DRIVE_ENGINE_IMPULSE: f32 = 8.0;
pub const ALIEN_SHIP_ROTATION_IMPULSE: f32 = 6.0 * ALIEN_SHIP_DRIVE_ENGINE_IMPULSE;
pub const ALIEN_SHIP_MASS: f32 = 1.0;

pub const ALIEN_SHIP_LASER_COOLDOWN_S: f32 = 0.33;

const MAX_SHOOT_THETA: f32 = PI / 16.0;

const ENABLE_SHOOTING: bool = true;

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
    settings: Res<GameSettings>,
    mut orientation_controller_queue: ResMut<AIControllerQueues>,
    time: Res<Time>,
    mut impulses: EventWriter<AddExternalImpulse>,
    player: Query<&Transform, With<PlayerMarker>>,
    mut query: Query<
        (
            Entity,
            &Transform,
            &Velocity,
            &OrientationController,
            &PositionController,
            &mut LaserAbility,
            &mut Thruster,
        ),
        With<AlienShipMarker>,
    >,
    camera: Query<(&Transform, &OrthographicProjection), With<GameCameraMarker>>,
) {
    // The actual AI is going to be a bit tricky.
    // We'll at least have to implement a basic PID control loop.

    // For now, it is very dumb. It aims at the player and accelerates if it points in kinda in the player direction.
    if let Ok((cam_transform, cam_proj)) = camera.get_single() {
        let cam_pos = cam_transform.translation.xy();
        let cam_area = cam_proj.area;
        let abs_cam_area = Rect {
            min: cam_area.min + cam_pos,
            max: cam_area.max + cam_pos,
        };
        if let Ok(player_t) = player.get_single() {
            for (
                entity,
                t,
                v,
                orientation_controller,
                position_controller,
                mut laser_ability,
                mut thruster,
            ) in query.iter_mut()
            {
                let mut angular_impulse = 0.0;

                let local_forward = t.up().xy();
                let d = (player_t.translation - t.translation).xy();
                let orientation_to_player = local_forward.angle_between(d);
                if ENABLE_SHOOTING
                    && d.length() < AGGRO_RANGE
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

                let mut request_dynamics_controllers_update = false;
                let (cmd_torque, cmd_end_time) = orientation_controller.current_command;
                if time.elapsed_seconds() < cmd_end_time && cmd_torque.abs() > 0.01 {
                    angular_impulse += cmd_torque;
                    let particle_distance = 24.0;
                    let xy = t.translation.xy();
                    if abs_cam_area.contains(xy) {
                        spawn_rotation_thruster_cone(
                            &mut commands,
                            settings.entities_quantity,
                            &time,
                            xy + t.right().xy().normalize()
                                * particle_distance
                                * cmd_torque.signum(),
                            v.linvel,
                            t.down().xy().normalize(),
                        );
                    }
                } else if time.elapsed_seconds() > cmd_end_time {
                    request_dynamics_controllers_update = true;
                }

                let (cmd_thrust, cmd_end_time) = position_controller.current_command;
                if time.elapsed_seconds() < cmd_end_time && cmd_thrust.abs() > 0.01 {
                    thruster.throttle(time.delta_seconds());
                } else if time.elapsed_seconds() > cmd_end_time {
                    thruster.release(time.delta_seconds());
                    request_dynamics_controllers_update = true;
                } else {
                    thruster.release(time.delta_seconds());
                }

                if request_dynamics_controllers_update {
                    orientation_controller_queue.controllers.push_back(entity);
                }
                impulses.send(AddExternalImpulse {
                    entity,
                    impulse: Vec2::ZERO,
                    torque_impulse: angular_impulse
                        * time.delta_seconds()
                        * GLOBAL_IMPULSE_DURATION_MULT,
                });
            }
        } else {
            error!("Can't process Alien Ships: player not found");
        }
    }
}

pub fn cleanup(mut commands: Commands, ships: Query<Entity, With<AlienShipMarker>>) {
    for entity in ships.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
