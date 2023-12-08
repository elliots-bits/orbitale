use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum AppStage {
    AI,
    Control,
    Trajectories,
    Simulation,
    AggregateImpulses,
    Draw,
    DespawnQueue,
}

pub fn setup(app: &mut App) {
    app.configure_sets(
        Update,
        (
            AppStage::AI,
            AppStage::Control,
            AppStage::Simulation,
            // PhysicsSet::SyncBackend,
            // PhysicsSet::StepSimulation,
            // PhysicsSet::Writeback,
            AppStage::AggregateImpulses,
            AppStage::Draw,
            AppStage::Trajectories,
            AppStage::DespawnQueue,
        )
            .chain(),
    );
}
