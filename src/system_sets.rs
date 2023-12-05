use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum AppStage {
    Simulation,
    AggregateImpulses,
    Draw,
}

pub fn setup(app: &mut App) {
    app.configure_sets(
        Update,
        (
            AppStage::Simulation,
            AppStage::AggregateImpulses,
            AppStage::Draw,
        )
            .chain(),
    );
}
