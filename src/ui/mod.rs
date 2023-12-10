use bevy::prelude::*;

mod death_screen;
mod diagnostics;
mod healthbar;
mod menu;
pub mod radar;
mod score;

pub use menu::{Difficulty, EntitiesQuantity, GameSettings};
pub use score::Score;

pub fn setup(app: &mut App) {
    menu::setup(app);
    score::setup(app);
    death_screen::setup(app);
    radar::setup(app);
    healthbar::setup(app);
    diagnostics::setup(app);
}
