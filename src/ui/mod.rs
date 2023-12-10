use bevy::prelude::*;

mod death_screen;
mod entities_count;
mod healthbar;
mod menu;
mod radar;
mod score;

pub use menu::{Difficulty, GameSettings};
pub use score::Score;

pub fn setup(app: &mut App) {
    menu::setup(app);
    score::setup(app);
    death_screen::setup(app);
    radar::setup(app);
    healthbar::setup(app);
    entities_count::setup(app);
}
