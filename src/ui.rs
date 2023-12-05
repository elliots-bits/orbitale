use bevy::{prelude::*, render::view::RenderLayers, window::PrimaryWindow};
use bevy_rapier2d::dynamics::Velocity;
use bevy_vector_shapes::{
    painter::ShapePainter,
    shapes::{DiscPainter, RectPainter},
};

use crate::{
    alien_ship::AlienShipMarker,
    camera::{GameCameraMarker, UI_LAYER},
    player::{PlayerHP, PlayerMarker},
};

const BAR_SIZE: Vec2 = Vec2 { x: 300.0, y: 25.0 };
const RADAR_HUD_RADIUS: f32 = 150.0;

pub fn draw_healthbar(
    mut painter: ShapePainter,
    q_window: Query<&Window, With<PrimaryWindow>>,
    player_hp: Query<&PlayerHP, With<PlayerMarker>>,
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

pub fn draw_hud(
    mut painter: ShapePainter,
    camera: Query<(&Transform, &OrthographicProjection), With<GameCameraMarker>>,
    player: Query<(&Transform, &Velocity), With<PlayerMarker>>,
    alien_ships: Query<(&Transform, &Velocity), With<AlienShipMarker>>,
) {
    if let Ok((pt, pv)) = player.get_single() {
        // Draw Radar circle
        painter.set_2d();
        painter.render_layers = Some(RenderLayers::layer(UI_LAYER));
        painter.color = Color::Rgba {
            red: 0.25,
            green: 0.25,
            blue: 0.25,
            alpha: 0.2,
        };
        painter.hollow = true;
        painter.thickness = 1.0;
        painter.circle(RADAR_HUD_RADIUS);

        if let Ok((cam_transform, cam_proj)) = camera.get_single() {
            let cam_pos = cam_transform.translation.xy();
            let cam_area = cam_proj.area;
            let abs_cam_area = Rect {
                min: cam_area.min + cam_pos,
                max: cam_area.max + cam_pos,
            };
            for (at, av) in alien_ships.iter() {
                if abs_cam_area.contains(at.translation.xy()) {
                    let dp = at.translation.xy() - pt.translation.xy();
                    let dv = av.linvel - pv.linvel;
                }
            }
        }
    }
}
