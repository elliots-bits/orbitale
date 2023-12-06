use bevy::prelude::*;

#[derive(Component)]
pub struct HealthPoints {
    pub max: f32,
    pub current: f32,
}

impl HealthPoints {
    pub fn decrease(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }
}
