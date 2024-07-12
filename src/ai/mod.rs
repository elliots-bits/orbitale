pub mod orientation_controller;
pub mod position_controller;

use std::{collections::VecDeque, f32::consts::PI};

use bevy::prelude::*;
use bevy_rapier2d::dynamics::Velocity;

use crate::{
    alien_ship::AlienShipMarker, course_planner::ComputedTrajectory, gravity::AffectedByGravity,
    player::PlayerMarker,
};

use self::{
    orientation_controller::{OrientationController, MIN_ROTATION_THETA},
    position_controller::PositionController,
};

const MAX_AI_STATE_UPDATES_PER_FRAME: u32 = 1000;
const MAX_DYNAMICS_CONTROLLERS_UPDATES_PER_FRAME: u32 = 250;
const MIN_DELTA_V: f32 = 25.0;
const MATCH_DELTA_V_THRESHOLD: f32 = 3000.0;
pub const AGGRO_RANGE: f32 = 1500.0;

#[derive(Resource, Default)]
pub struct AIControllerQueues {
    pub controllers: VecDeque<Entity>,
    pub ai_state: VecDeque<Entity>,
}

impl AIControllerQueues {
    pub fn queue_spawned(&mut self, entity: Entity) {
        self.ai_state.push_front(entity);
        self.controllers.push_front(entity);
    }
}

#[derive(Component, Default)]
pub struct ShipAi {
    pub state: AiState,
}

#[derive(Default, Debug, PartialEq, Eq)]
pub enum AiState {
    #[default]
    Aggro,
    Intercept,
    MatchVelocities,
    AvoidCrash,
}

pub fn setup(mut commands: Commands) {
    commands.insert_resource(AIControllerQueues::default());
}

pub fn update_ai_states(
    player: Query<(&Transform, &Velocity), With<PlayerMarker>>,
    mut queue: ResMut<AIControllerQueues>,
    mut ships: Query<
        (&Transform, &Velocity, &ComputedTrajectory, &mut ShipAi),
        With<AlienShipMarker>,
    >,
) {
    if let Ok((player_transform, player_velocity)) = player.get_single() {
        let mut to_push_back = Vec::<Entity>::new();
        let mut updated_controllers = 0;
        while let Some(enemy_entity) = queue.ai_state.pop_front() {
            // We limit the number of controller updates per frame to limit performance impact.
            // Controllers that were not updated stay in the queue for the next frames.
            if updated_controllers < MAX_AI_STATE_UPDATES_PER_FRAME {
                if let Ok((enemy_transform, enemy_velocity, enemy_trajectory, mut ai_controller)) =
                    ships.get_mut(enemy_entity)
                {
                    // We update the AI controller state according to our relative position & velocity to the player,
                    // and according to our current trajectory w.r.t celestial bodies.
                    let relative_position =
                        player_transform.translation.xy() - enemy_transform.translation.xy();
                    let relative_velocity = player_velocity.linvel - enemy_velocity.linvel;

                    if relative_position.length() < AGGRO_RANGE {
                        // We are near the player and can start attacking it.
                        ai_controller.state = AiState::Aggro;
                    } else if enemy_trajectory.closest_flyby < 32.0 {
                        // We are on a collision course with a celestial body and need to avoid crashing.
                        ai_controller.state = AiState::AvoidCrash;
                    } else if ai_controller.state != AiState::Intercept
                        && relative_velocity.length() > MATCH_DELTA_V_THRESHOLD
                    {
                        // The player is getting away fast and we need to catch up.
                        ai_controller.state = AiState::MatchVelocities;
                    } else {
                        // The player is at a appreciable distance and we need to get closer.
                        ai_controller.state = AiState::Intercept;
                    }
                    // debug!("Set ai state to {:?}", ai.state);
                    updated_controllers += 1;
                    to_push_back.push(enemy_entity);
                }
            } else {
                break;
            }
        }
        for entity in to_push_back.iter() {
            queue.ai_state.push_back(*entity);
        }
    }
}

pub fn update_ai_controllers(
    time: Res<Time>,
    player: Query<(&Transform, &Velocity), With<PlayerMarker>>,
    mut queue: ResMut<AIControllerQueues>,
    mut ships: Query<
        (
            &Transform,
            &Velocity,
            &AffectedByGravity,
            &ShipAi,
            &mut OrientationController,
            &mut PositionController,
        ),
        With<AlienShipMarker>,
    >,
) {
    if let Ok((player_transform, player_velocity)) = player.get_single() {
        let mut updated_controllers = 0;
        while let Some(enemy_entity) = queue.controllers.pop_front() {
            // We limit the number of controller updates per frame to limit performance impact.
            // Controllers that were not updated stay in the queue for the next frames.

            if updated_controllers < MAX_DYNAMICS_CONTROLLERS_UPDATES_PER_FRAME {
                if let Ok((
                    enemy_transform,
                    enemy_velocity,
                    gravity,
                    ai_controller,
                    mut orientation_controller,
                    mut position_controller,
                )) = ships.get_mut(enemy_entity)
                {
                    let local_forward = enemy_transform.up().xy();
                    let relative_position =
                        (player_transform.translation - enemy_transform.translation).xy();
                    let relative_velocity = player_velocity.linvel - enemy_velocity.linvel;
                    let angle_to_player: f32 = relative_position.y.atan2(relative_position.x);
                    let current_orientation = local_forward.y.atan2(local_forward.x);
                    match ai_controller.state {
                        AiState::Aggro => {
                            orientation_controller.target(angle_to_player);
                            orientation_controller.update_command(
                                &time,
                                current_orientation,
                                enemy_velocity.angvel,
                            );
                            position_controller.sleep(&time, 0.2);
                        }
                        AiState::Intercept => {
                            // Either aim towards towards player and accelerate to put on intercept course,
                            // Or turn around and brake in order to stop near the player.

                            let speed_dot = relative_velocity.length()
                                * (-relative_velocity.normalize_or_zero())
                                    .dot(relative_position.normalize_or_zero());

                            let should_brake = position_controller.should_brake(
                                (relative_position.length() - AGGRO_RANGE * 0.5).max(0.0),
                                speed_dot,
                            );

                            if speed_dot > 0.25 && should_brake {
                                // Our trajectory is aligned with the player's and we need to start reducing relative velocity.
                                orientation_controller
                                    .target(relative_velocity.y.atan2(relative_velocity.x));
                                orientation_controller.update_command(
                                    &time,
                                    current_orientation,
                                    enemy_velocity.angvel,
                                );
                                if orientation_controller
                                    .at_target(current_orientation, MIN_ROTATION_THETA * 2.0)
                                {
                                    // We are facing opposite direction to our relative velocity and can thrust to brake.
                                    position_controller.accelerate(&time, 0.05);
                                }
                            } else {
                                // Our trajectory is not aligned with the player and we need to adjust it.
                                let wanted_dv = Vec2 {
                                    x: angle_to_player.cos(),
                                    y: angle_to_player.sin(),
                                }
                                .normalize()
                                    * MATCH_DELTA_V_THRESHOLD;

                                // We compute the direction we should face to match player trajectory.
                                let drift = -relative_velocity - wanted_dv;
                                let direction = -drift;
                                let orientation = direction.y.atan2(direction.x);

                                orientation_controller.target(orientation);
                                orientation_controller.update_command(
                                    &time,
                                    current_orientation,
                                    enemy_velocity.angvel,
                                );
                                if orientation_controller
                                    .at_target(current_orientation, MIN_ROTATION_THETA)
                                    && drift.length() > MIN_DELTA_V
                                {
                                    // We are aligned with desired trajectory and can start accelerating.
                                    position_controller.accelerate(&time, 0.05);
                                } else {
                                    // We are not aligned or already going fast enough towards desired trajectory.
                                    // We do not accelerate.
                                    position_controller.sleep(&time, 0.05);
                                }
                            }
                        }
                        AiState::MatchVelocities => {
                            // Our ship needs to match player velocity.
                            // We face the relative velocity direction and accelerate.
                            let orientation = relative_velocity.y.atan2(relative_velocity.x);
                            orientation_controller.target(orientation);
                            orientation_controller.update_command(
                                &time,
                                current_orientation,
                                enemy_velocity.angvel,
                            );
                            if orientation_controller.at_target(current_orientation, PI / 8.0) {
                                let tts =
                                    position_controller.time_to_stop(relative_velocity.length());
                                position_controller
                                    .accelerate(&time, (tts / 2.0 - 0.1).max(0.01).min(0.25));
                            }
                        }
                        AiState::AvoidCrash => {
                            // We are on a collision course with a celestial body.
                            // We need to aim for an escape trajectory facing away from the current gravity vector we are experiencing.
                            let escape_vector = (-Vec2::Y
                                .rotate(gravity.last_acceleration.normalize_or_zero())
                                - enemy_velocity.linvel.normalize_or_zero())
                            .normalize_or_zero();
                            let escape_orientation = escape_vector.y.atan2(escape_vector.x);
                            orientation_controller.target(escape_orientation);
                            orientation_controller.update_command(
                                &time,
                                current_orientation,
                                enemy_velocity.angvel,
                            );
                            if orientation_controller.at_target(current_orientation, PI / 4.0) {
                                position_controller.accelerate(&time, 1.0);
                            }
                        }
                    };
                    updated_controllers += 1;
                }
            } else {
                break;
            }
        }
    }
}
