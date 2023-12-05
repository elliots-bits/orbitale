use std::{collections::HashMap, ops::Add};

use bevy::prelude::*;
use bevy_rapier2d::prelude::ExternalImpulse;

use crate::system_sets::AppStage;

pub fn setup(app: &mut App) {
    app.init_resource::<Events<AddExternalImpulse>>(); // Override Bevy's automatic event's cleanup. We drain these events ourselves.
    app.add_systems(
        Update,
        drain_added_external_impulses.in_set(AppStage::AggregateImpulses),
    );
}

fn drain_added_external_impulses(
    mut commands: Commands,
    mut added_impulses: ResMut<Events<AddExternalImpulse>>,
) {
    let mut entity_total_frame_impulse: HashMap<Entity, ExternalImpulse> =
        HashMap::<Entity, ExternalImpulse>::new();
    for added_impulse in added_impulses.drain() {
        entity_total_frame_impulse
            .entry(added_impulse.entity)
            .and_modify(|e| *e = *e + added_impulse)
            .or_insert(added_impulse.into());
    }
    for (&entity, &impulse) in entity_total_frame_impulse.iter() {
        if let Some(mut entity) = commands.get_entity(entity) {
            entity.insert(impulse);
        }
    }
}

#[derive(Event, Clone, Copy, Debug)]
pub struct AddExternalImpulse {
    pub entity: Entity,
    pub impulse: Vec2,
    pub torque_impulse: f32,
}

impl From<AddExternalImpulse> for ExternalImpulse {
    fn from(value: AddExternalImpulse) -> Self {
        Self {
            impulse: value.impulse,
            torque_impulse: value.torque_impulse,
        }
    }
}

impl Add<AddExternalImpulse> for ExternalImpulse {
    type Output = Self;

    fn add(self, rhs: AddExternalImpulse) -> Self::Output {
        Self {
            impulse: self.impulse + rhs.impulse,
            torque_impulse: self.torque_impulse + rhs.torque_impulse,
        }
    }
}
