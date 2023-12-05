use bevy::{prelude::*, utils::HashSet};

use crate::system_sets::AppStage;

#[derive(Resource, Default, Clone)]
pub struct DespawnQueue(pub HashSet<Entity>, pub HashSet<Entity>);

pub fn despawn_entities(mut commands: Commands, mut queue: ResMut<DespawnQueue>) {
    for entity in queue.0.iter() {
        if commands.get_entity(*entity).is_some() {
            commands.entity(*entity).despawn_recursive();
        }
    }
    queue.0 = queue.1.clone();
    queue.1 = HashSet::new();
}

pub fn setup(app: &mut App) {
    app.insert_resource(DespawnQueue::default());
    app.add_systems(Update, despawn_entities.in_set(AppStage::DespawnQueue));
}
