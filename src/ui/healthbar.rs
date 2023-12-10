use std::f32::consts::PI;

use bevy::{prelude::*, render::view::RenderLayers, window::PrimaryWindow};
use bevy_rapier2d::{dynamics::Velocity, geometry::Collider};
use bevy_vector_shapes::{
    painter::ShapePainter,
    shapes::{Cap, DiscPainter, LinePainter, RectPainter},
};
use colorgrad::CustomGradient;

use crate::{
    alien_ship::AlienShipMarker,
    camera::{GameCameraMarker, UI_LAYER},
    celestial_body::CelestialBodyMarker,
    course_planner::{ComputedTrajectory, PLAYER_PLAN_DURATION, PLAYER_PLAN_STEP_DT},
    healthpoints::HealthPoints,
    player::PlayerMarker,
};

const BAR_SIZE: Vec2 = Vec2 { x: 300.0, y: 25.0 };

pub fn setup(app: &mut App) {
    app.add_systems(Update, draw_healthbar);
}

fn draw_healthbar(
    mut painter: ShapePainter,
    q_window: Query<&Window, With<PrimaryWindow>>,
    player_hp: Query<&HealthPoints, With<PlayerMarker>>,
) {
    let win = q_window.single();
    if let Ok(hp) = player_hp.get_single() {
        painter.set_2d();

        // Health bar
        let hp_frac = hp.current / hp.max;
        let fill_width = (BAR_SIZE.x - 4.0) * hp_frac;
        let x_offset = -(BAR_SIZE.x - 4.0 - fill_width) / 2.0;
        painter.set_translation(Vec3::new(
            x_offset,
            -win.height() / 2.0 + BAR_SIZE.y / 2.0 + 20.0,
            0.0,
        ));
        painter.render_layers = Some(RenderLayers::layer(UI_LAYER));
        painter.color = Color::rgba((1.0 - hp_frac).powf(0.5), hp_frac.powf(0.5), 0.0, 1.0);
        painter.corner_radii = Vec4::splat(0.0);
        painter.hollow = false;
        painter.rect(Vec2 {
            x: fill_width,
            y: BAR_SIZE.y - 2.0,
        });

        // Outline
        painter.set_translation(Vec3::new(
            0.0,
            -win.height() / 2.0 + BAR_SIZE.y / 2.0 + 20.0,
            0.0,
        ));
        painter.render_layers = Some(RenderLayers::layer(UI_LAYER));
        painter.color = Color::WHITE;
        painter.corner_radii = Vec4::splat(5.0);
        painter.hollow = true;
        painter.thickness = 2.0;
        painter.rect(BAR_SIZE);
    }
}
