use bevy::prelude::*;
use bevy_rapier2d::plugin::PhysicsSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum AppStage {
    Control,
    Simulation,
    AggregateImpulses,
    Draw,
    DespawnQueue,
}

pub fn setup(app: &mut App) {
    app.configure_sets(
        Update,
        (
            AppStage::Control,
            AppStage::Simulation,
            PhysicsSet::SyncBackend,
            PhysicsSet::StepSimulation,
            PhysicsSet::Writeback,
            AppStage::AggregateImpulses,
            AppStage::Draw,
            AppStage::DespawnQueue,
        )
            .chain(),
    );
}
