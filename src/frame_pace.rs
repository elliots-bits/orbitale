use bevy::prelude::*;
use bevy_framepace::{FramepaceSettings, Limiter};

pub fn setup(app: &mut App) {
    app.add_plugins(bevy_framepace::FramepacePlugin);
    app.add_systems(Update, update_frame_pace);
}

fn update_frame_pace(mut settings: ResMut<FramepaceSettings>, keys: Res<Input<KeyCode>>) {
    if keys.pressed(KeyCode::F1) {
        settings.limiter = Limiter::from_framerate(20.0);
    }
    if keys.pressed(KeyCode::F2) {
        settings.limiter = Limiter::from_framerate(30.0);
    }
    if keys.pressed(KeyCode::F3) {
        settings.limiter = Limiter::from_framerate(60.0);
    }
    if keys.pressed(KeyCode::F4) {
        settings.limiter = Limiter::Auto;
    }
}
