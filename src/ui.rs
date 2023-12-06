use bevy::{prelude::*, render::view::RenderLayers, window::PrimaryWindow};
use bevy_rapier2d::{
    dynamics::Velocity,
    geometry::{Collider, ColliderMassProperties},
};
use bevy_vector_shapes::{
    painter::ShapePainter,
    shapes::{DiscPainter, LinePainter, RectPainter, ThicknessType},
};
use colorgrad::CustomGradient;

use crate::{
    alien_ship::AlienShipMarker,
    camera::{GameCameraMarker, UI_LAYER},
    celestial_body::CelestialBodyMarker,
    gravity::plan_course,
    healthpoints::HealthPoints,
    player::PlayerMarker,
};

const BAR_SIZE: Vec2 = Vec2 { x: 300.0, y: 25.0 };
const RADAR_HUD_INNER_RADIUS: f32 = 150.0;
const RADAR_HUD_OUTER_RADIUS: f32 = 400.0;
const RADAR_HUD_SCALE: f32 = 1.0 / 80.0;
const RADAR_COLOR_MAX_SPEED: f32 = 3000.0;
const RADAR_ENTITIES_ALPHA: f32 = 0.9;
const RADAR_CIRCLES_ALPHA: f32 = 0.6;

#[derive(Resource)]
pub struct RadarShipsColorGradient(pub colorgrad::Gradient);

pub fn setup(mut commands: Commands) {
    commands.insert_resource(RadarShipsColorGradient(
        CustomGradient::new()
            .colors(&[
                colorgrad::Color::new(0.0, 0.3, 1.0, 1.0),
                colorgrad::Color::new(0.6, 0.6, 0.6, 1.0),
                colorgrad::Color::new(1.0, 0.0, 0.0, 1.0),
            ])
            .build()
            .unwrap(),
    ));
}

pub fn draw_healthbar(
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

pub fn draw_hud(
    mut painter: ShapePainter,
    ship_color_gradient: Res<RadarShipsColorGradient>,
    camera: Query<(&Transform, &OrthographicProjection), With<GameCameraMarker>>,
    player: Query<(&Transform, &Velocity), With<PlayerMarker>>,
    alien_ships: Query<(&Transform, &Velocity), With<AlienShipMarker>>,
    celestial_bodies: Query<
        (&Transform, &ColliderMassProperties, &Collider),
        With<CelestialBodyMarker>,
    >,
) {
    if let Ok((pt, pv)) = player.get_single() {
        // Draw Radar circles
        painter.set_2d();
        painter.render_layers = Some(RenderLayers::layer(UI_LAYER));
        painter.color = Color::Rgba {
            red: 0.4,
            green: 0.4,
            blue: 0.4,
            alpha: RADAR_CIRCLES_ALPHA,
        };
        painter.hollow = true;
        painter.thickness = 1.0;
        painter.circle(RADAR_HUD_INNER_RADIUS);
        painter.circle(RADAR_HUD_OUTER_RADIUS);

        // Draw current orientation line
        let fwd = pt.up().xy();
        let theta = fwd.y.atan2(fwd.x);
        painter.line(
            Vec3::new(theta.cos() * 32.0, theta.sin() * 32.0, 0.0),
            Vec3::new(
                theta.cos() * RADAR_HUD_INNER_RADIUS,
                theta.sin() * RADAR_HUD_INNER_RADIUS,
                0.0,
            ),
        );

        // Draw planned course
        let bodies = celestial_bodies
            .iter()
            .map(|(transform, massprops, collider)| {
                if let ColliderMassProperties::Mass(mass) = massprops {
                    (
                        transform.translation.xy(),
                        *mass,
                        collider.as_ball().unwrap().radius(),
                    )
                } else {
                    panic!("Use ColliderMassProperties::Mass(x) for celestial bodies")
                }
            })
            .collect::<Vec<(Vec2, f32, f32)>>();
        let planned_course = plan_course(60.0, 0.5, pt.translation.xy(), pv.linvel, bodies);

        if planned_course.closest_flyby <= 0.0 {
            painter.color = Color::RED;
        } else {
            painter.color = Color::GREEN;
        }
        painter.thickness = 0.5;
        let mut last_point: Option<Vec2> = None;
        for &point in planned_course.path.iter() {
            if let Some(previous_point) = last_point {
                let a = (previous_point - pt.translation.xy()) * RADAR_HUD_SCALE;
                let b = (point - pt.translation.xy()) * RADAR_HUD_SCALE;
                if a.length() >= RADAR_HUD_INNER_RADIUS
                    && a.length() <= RADAR_HUD_OUTER_RADIUS
                    && b.length() >= RADAR_HUD_INNER_RADIUS
                    && b.length() <= RADAR_HUD_OUTER_RADIUS
                {
                    // debug!("Drawing line from {} to {}", a, b);
                    painter.line(a.extend(0.0), b.extend(0.0));
                }
            }
            last_point = Some(point);
        }
        debug!("");

        if let Ok((cam_transform, cam_proj)) = camera.get_single() {
            let cam_pos = cam_transform.translation.xy();
            let cam_area = cam_proj.area;
            let abs_cam_area = Rect {
                min: cam_area.min + cam_pos,
                max: cam_area.max + cam_pos,
            };
            for (at, av) in alien_ships.iter() {
                let dp = at.translation.xy() - pt.translation.xy();
                let dv = av.linvel - pv.linvel;

                let (theta, r) = (dp.y.atan2(dp.x), dp.length());
                let radar_r = ((r - RADAR_HUD_INNER_RADIUS) * RADAR_HUD_SCALE
                    + RADAR_HUD_INNER_RADIUS)
                    .clamp(RADAR_HUD_INNER_RADIUS, RADAR_HUD_OUTER_RADIUS);
                let ship_closing_speed = dv.length() * dv.normalize().dot(dp.normalize());

                let speed_color_interp = 1.0
                    - ((ship_closing_speed + RADAR_COLOR_MAX_SPEED)
                        / (RADAR_COLOR_MAX_SPEED * 2.0))
                        .clamp(0.0, 1.0);
                let ship_radar_color = ship_color_gradient.0.at(speed_color_interp.into());

                painter.set_translation(Vec3 {
                    x: theta.cos() * radar_r,
                    y: theta.sin() * radar_r,
                    z: 0.0,
                });
                painter.color = Color::rgba(
                    ship_radar_color.r as f32,
                    ship_radar_color.g as f32,
                    ship_radar_color.b as f32,
                    RADAR_ENTITIES_ALPHA,
                );
                painter.hollow = true;

                let max_radar_range = RADAR_HUD_OUTER_RADIUS / RADAR_HUD_SCALE;
                let size = ((max_radar_range - r) / max_radar_range).clamp(3.0, 6.0);

                painter.thickness = (size / 4.0).clamp(0.0, 1.0);
                painter.set_rotation(Quat::from_axis_angle(Vec3::Z, dv.y.atan2(dv.x)));
                painter.rect(Vec2::splat(size));

                if abs_cam_area.contains(at.translation.xy()) {
                    // Highlight ship
                    painter.set_translation(dp.extend(0.0) / cam_proj.scale);
                    painter.set_rotation(Quat::default());
                    painter.hollow = true;
                    painter.thickness = 1.0;
                    painter.color = Color::hex("ff3030ff").unwrap();
                    painter.rect(Vec2::splat((64.0 / cam_proj.scale).max(10.0)));
                }
            }

            for (at, _, collider) in celestial_bodies.iter() {
                let dp = at.translation.xy() - pt.translation.xy();
                let dv = -pv.linvel; // Todo if needed: take body velocity into account

                let (theta, r) = (dp.y.atan2(dp.x), dp.length());
                let radar_r = ((r - RADAR_HUD_INNER_RADIUS) * RADAR_HUD_SCALE
                    + RADAR_HUD_INNER_RADIUS)
                    .clamp(RADAR_HUD_INNER_RADIUS, RADAR_HUD_OUTER_RADIUS);
                let ship_closing_speed = dv.length() * dv.normalize().dot(dp.normalize());
                let size = collider.as_ball().unwrap().radius() / RADAR_HUD_SCALE;

                if radar_r - size >= RADAR_HUD_INNER_RADIUS
                    && radar_r + size <= RADAR_HUD_OUTER_RADIUS
                {
                    let speed_color_interp = 1.0
                        - ((ship_closing_speed + RADAR_COLOR_MAX_SPEED)
                            / (RADAR_COLOR_MAX_SPEED * 2.0))
                            .clamp(0.0, 1.0);
                    let ship_radar_color = ship_color_gradient.0.at(speed_color_interp.into());

                    painter.set_translation(Vec3 {
                        x: theta.cos() * radar_r,
                        y: theta.sin() * radar_r,
                        z: 0.0,
                    });
                    painter.color = Color::rgba(
                        ship_radar_color.r as f32,
                        ship_radar_color.g as f32,
                        ship_radar_color.b as f32,
                        RADAR_ENTITIES_ALPHA,
                    );

                    painter.hollow = false;
                    painter.set_rotation(Quat::from_axis_angle(Vec3::Z, dv.y.atan2(dv.x)));
                    painter.circle(size);
                }
            }
        }
    }
}
