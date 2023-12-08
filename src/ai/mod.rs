pub mod orientation_controller;

use bevy::prelude::*;

use self::orientation_controller::OrientationControllerQueue;

#[derive(Component, Default)]
pub struct ShipAi {
    pub state: AiState,
}

#[derive(Default)]
pub enum AiState {
    #[default]
    Aggro,
}

pub fn setup(mut commands: Commands) {
    commands.insert_resource(OrientationControllerQueue::default());
}
