pub mod orientation_controller;

use std::collections::VecDeque;

use bevy::prelude::*;
use bevy_rapier2d::dynamics::Velocity;

use crate::{alien_ship::AlienShipMarker, player::PlayerMarker};

use self::orientation_controller::OrientationController;

const MAX_CONTROLLER_UPDATES_PER_FRAME: u32 = 50;

#[derive(Resource, Default)]
pub struct OrientationControllerQueue(pub VecDeque<Entity>);

#[derive(Component, Default)]
pub struct ShipAi {
    pub state: AiState,
}

#[derive(Default)]
pub enum AiState {
    #[default]
    Aggro,
    MaintainDistance,
    Intercept,
    MatchVelocities,
}

pub fn setup(mut commands: Commands) {
    commands.insert_resource(OrientationControllerQueue::default());
}

pub fn update_ai_controllers(
    time: Res<Time>,
    player: Query<&Transform, With<PlayerMarker>>,
    mut queue: ResMut<OrientationControllerQueue>,
    mut ships: Query<
        (&Transform, &Velocity, &ShipAi, &mut OrientationController),
        With<AlienShipMarker>,
    >,
) {
    if let Ok(player_t) = player.get_single() {
        let mut updated_controllers = 0;
        while let Some(e) = queue.0.pop_front() {
            if updated_controllers < MAX_CONTROLLER_UPDATES_PER_FRAME {
                if let Ok((t, v, ai, mut controller)) = ships.get_mut(e) {
                    match ai.state {
                        AiState::Aggro => {
                            let local_forward = t.up().xy();
                            let d = (player_t.translation - t.translation).xy();
                            let angle_to_player = d.y.atan2(d.x);
                            let current_orientation = local_forward.y.atan2(local_forward.x);
                            controller.target(angle_to_player);
                            controller.update_command(&time, current_orientation, v.angvel);
                        }
                        AiState::MaintainDistance => {}
                        AiState::Intercept => {}
                        AiState::MatchVelocities => {}
                    };
                    updated_controllers += 1;
                }
            } else {
                break;
            }
        }
    }
}
