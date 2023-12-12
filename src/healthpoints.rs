use bevy::prelude::*;

use crate::ui::Difficulty;

#[derive(Component)]
pub struct HealthPoints {
    pub max: f32,
    pub current: f32,
}

impl HealthPoints {
    pub fn decrease(&mut self, amount: f32, difficulty: Difficulty) {
        let damage_multiplier = match difficulty {
            Difficulty::GodMode => 0.0,
            Difficulty::Easy => 0.25,
            Difficulty::Normal => 1.0,
            Difficulty::Hard => 2.0,
            Difficulty::Impossible => 3.0,
        };

        self.current = (self.current - (amount * damage_multiplier)).max(0.0);
    }
}
