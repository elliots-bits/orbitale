use bevy::prelude::*;

use crate::{
    alien_ship::AlienShipMarker, despawn_queue::DespawnQueue, healthpoints::HealthPoints,
    player::PlayerMarker, ui::Score, AppState,
};

pub fn update(
    mut despawn_queue: ResMut<DespawnQueue>,
    mut next_state: ResMut<NextState<AppState>>,
    player_hp: Query<&HealthPoints, With<PlayerMarker>>,
    alien_ships: Query<(Entity, &HealthPoints), With<AlienShipMarker>>,
    mut score: ResMut<Score>,
) {
    if let Ok(&HealthPoints { current, .. }) = player_hp.get_single() {
        if current <= 0.0 {
            next_state.set(AppState::DeathScreen);
        }
    }
    for (entity, &HealthPoints { current, .. }) in alien_ships.iter() {
        if current <= 0.0 {
            despawn_queue.1.insert(entity);
            score.enemies_killed += 1;
        }
    }
}
