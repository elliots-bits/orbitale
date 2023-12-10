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
    orientation_controller::OrientationController, position_controller::PositionController,
};

const MAX_AI_STATE_UPDATES_PER_FRAME: u32 = 1000;
const MAX_DYNAMICS_CONTROLLERS_UPDATES_PER_FRAME: u32 = 250;
const MIN_DELTA_V: f32 = 50.0;
const MATCH_DELTA_V_THRESHOLD: f32 = 5000.0;
const AGGRO_RANGE: f32 = 1250.0;

#[derive(Resource, Default)]
pub struct AIControllerQueues {
    pub controllers: VecDeque<Entity>,
    pub ai_state: VecDeque<Entity>,
}

impl AIControllerQueues {
    pub fn queue_spawned(&mut self, entity: Entity) {
        self.ai_state.push_back(entity);
        self.controllers.push_back(entity);
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
    if let Ok((pt, pv)) = player.get_single() {
        let mut to_push_back = Vec::<Entity>::new();
        let mut updated_controllers = 0;
        while let Some(e) = queue.ai_state.pop_front() {
            if updated_controllers < MAX_AI_STATE_UPDATES_PER_FRAME {
                if let Ok((t, v, traj, mut ai)) = ships.get_mut(e) {
                    let dp = pt.translation.xy() - t.translation.xy();
                    let dv = pv.linvel - v.linvel;

                    if dp.length() < AGGRO_RANGE {
                        ai.state = AiState::Aggro;
                    } else if traj.closest_flyby < 32.0 {
                        ai.state = AiState::AvoidCrash;
                    } else if ai.state != AiState::Intercept
                        && dv.length() > MATCH_DELTA_V_THRESHOLD
                    {
                        ai.state = AiState::MatchVelocities;
                    } else {
                        ai.state = AiState::Intercept;
                    }
                    // debug!("Set ai state to {:?}", ai.state);
                    updated_controllers += 1;
                    to_push_back.push(e);
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
    if let Ok((player_t, player_v)) = player.get_single() {
        let mut updated_controllers = 0;
        while let Some(e) = queue.controllers.pop_front() {
            if updated_controllers < MAX_DYNAMICS_CONTROLLERS_UPDATES_PER_FRAME {
                if let Ok((t, v, grav, ai, mut o_controller, mut p_controller)) = ships.get_mut(e) {
                    let local_forward = t.up().xy();
                    let d = (player_t.translation - t.translation).xy();
                    let dv = player_v.linvel - v.linvel;
                    let angle_to_player = d.y.atan2(d.x);
                    let current_orientation = local_forward.y.atan2(local_forward.x);
                    match ai.state {
                        AiState::Aggro => {
                            o_controller.target(angle_to_player);
                            o_controller.update_command(&time, current_orientation, v.angvel);
                            p_controller.sleep(&time, 0.5);
                        }
                        AiState::Intercept => {
                            // Either aim towards towards player to accelerate,
                            // Or in the opposite direction to decelerate.
                            // debug!("-------");
                            // debug!("d={}", d);
                            // debug!("dv={}", dv);
                            let speed_dot = dv.length() * (-dv.normalize()).dot(d.normalize());
                            // debug!("speed_dot={}", speed_dot);
                            let should_brake = p_controller.should_brake(d.length(), speed_dot);
                            // debug!("should_brake={}", should_brake);

                            if should_brake {
                                // debug!("Braking");
                                o_controller.target(dv.y.atan2(dv.x) + 2.0 * PI);
                                o_controller.update_command(&time, current_orientation, v.angvel);
                                if o_controller.at_target(current_orientation, PI / 8.0) {
                                    p_controller.accelerate(&time, 0.05);
                                }
                            } else {
                                let wanted_dv = Vec2 {
                                    x: angle_to_player.cos(),
                                    y: angle_to_player.sin(),
                                }
                                .normalize()
                                    * MATCH_DELTA_V_THRESHOLD;

                                let drift = -dv - wanted_dv;
                                // debug!("wanted dv: {}", wanted_dv);
                                // debug!("drift: {}", drift);
                                let direction = -drift;
                                let orientation = direction.y.atan2(direction.x);
                                // debug!("aiming at: {}", orientation);
                                o_controller.target(orientation);
                                o_controller.update_command(&time, current_orientation, v.angvel);
                                if o_controller.at_target(current_orientation, PI / 8.0)
                                    && drift.length() > MIN_DELTA_V
                                {
                                    // debug!("accelerating");
                                    p_controller.accelerate(&time, 0.05);
                                } else {
                                    // debug!("we're fast enough");
                                    p_controller.sleep(&time, 0.05);
                                }
                            }
                        }
                        AiState::MatchVelocities => {
                            let orientation = dv.y.atan2(dv.x);
                            o_controller.target(orientation);
                            o_controller.update_command(&time, current_orientation, v.angvel);
                            if o_controller.at_target(current_orientation, PI / 16.0) {
                                let tts = p_controller.time_to_stop(dv.length());
                                p_controller
                                    .accelerate(&time, (tts / 2.0 - 0.1).max(0.01).min(0.25));
                            }
                        }
                        AiState::AvoidCrash => {
                            let escape_vector = (-grav.last_acceleration.normalize_or_zero()
                                + v.linvel.normalize_or_zero())
                            .normalize_or_zero();
                            let escape_orientation = escape_vector.y.atan2(escape_vector.x);
                            o_controller.target(escape_orientation);
                            o_controller.update_command(&time, current_orientation, v.angvel);
                            if o_controller.at_target(current_orientation, PI / 4.0) {
                                p_controller.accelerate(&time, 1.0);
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
