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
const RADAR_HUD_INNER_RADIUS: f32 = 150.0;
const RADAR_HUD_OUTER_RADIUS: f32 = 400.0;
const RADAR_HUD_SCALE: f32 = 1.0 / 400.0;
const RADAR_COLOR_MAX_SPEED: f32 = 3000.0;
const RADAR_ENTITIES_ALPHA: f32 = 0.9;
const RADAR_CELESTIAL_BODIES_ALPHA: f32 = 0.7;
const RADAR_CIRCLES_ALPHA: f32 = 0.6;
const RADAR_PLANNED_COURSE_ALPHA: f32 = 0.4;

#[derive(Resource)]
pub struct RadarShipsColorGradient(pub colorgrad::Gradient);

#[derive(Resource)]
pub struct CoursePlanningColorGradient(pub colorgrad::Gradient);

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
    commands.insert_resource(CoursePlanningColorGradient(
        CustomGradient::new()
            .colors(&[
                colorgrad::Color::new(0.0, 1.0, 0.0, 1.0),
                colorgrad::Color::new(0.6, 0.5, 0.0, 1.0),
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
    course_color_gradient: Res<CoursePlanningColorGradient>,
    camera: Query<(&Transform, &OrthographicProjection), With<GameCameraMarker>>,
    player: Query<(&Transform, &Velocity, &ComputedTrajectory), With<PlayerMarker>>,
    alien_ships: Query<(&Transform, &Velocity), With<AlienShipMarker>>,
    celestial_bodies: Query<(&Transform, &Collider), With<CelestialBodyMarker>>,
) {
    if let Ok((pt, pv, traj)) = player.get_single() {
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
                let radar_r = world_to_radar(r);

                let ship_closing_speed =
                    dv.length() * dv.normalize_or_zero().dot(dp.normalize_or_zero());

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

            for (at, collider) in celestial_bodies.iter() {
                let dp = at.translation.xy() - pt.translation.xy();
                let dv = -pv.linvel; // Todo if needed: take body velocity into account

                let (theta, r) = (dp.y.atan2(dp.x), dp.length());
                let radar_r = world_to_radar(r);
                let body_radius = collider.as_ball().unwrap().radius();

                let body_closing_speed = dv.length() * dv.normalize().dot(dp.normalize());

                let closest_arc = radar_arc_at_distance(body_radius, r - body_radius / 2.0);

                if (RADAR_HUD_INNER_RADIUS..=RADAR_HUD_OUTER_RADIUS).contains(&radar_r) {
                    let speed_color_interp = 1.0
                        - ((body_closing_speed + RADAR_COLOR_MAX_SPEED)
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
                        RADAR_CELESTIAL_BODIES_ALPHA,
                    );

                    painter.set_translation(Vec3::ZERO);
                    painter.set_rotation(Quat::default());
                    painter.hollow = true;
                    // painter.thickness = radar_size_at_distance(body_radius, r);
                    painter.thickness = (body_radius * RADAR_HUD_SCALE) + 4.0;

                    let cap_radius = painter.thickness / 2.0;
                    let arc_covered_by_cap = radar_arc_at_distance(cap_radius, radar_r);
                    let (cap, arc) = if arc_covered_by_cap > PI / 8.0 {
                        (Cap::None, closest_arc)
                    } else {
                        (
                            Cap::Round,
                            (closest_arc - arc_covered_by_cap).max(PI / 64.0),
                        )
                    };
                    painter.cap = cap;
                    painter.arc(
                        radar_r.max(RADAR_HUD_INNER_RADIUS + painter.thickness),
                        -theta - arc + PI / 2.0,
                        -theta + arc + PI / 2.0,
                    );
                }
            }
        }
        // Draw planned course
        let max_segments = PLAYER_PLAN_DURATION / PLAYER_PLAN_STEP_DT;

        painter.set_translation(Vec2::ZERO.extend(0.5));
        painter.set_rotation(Quat::default());
        let mut last_point: Option<Vec2> = None;
        for (i, &(point, d)) in traj.path.iter().enumerate() {
            if let Some(previous_point) = last_point {
                let a = vec_to_radar(previous_point - pt.translation.xy());
                let b = vec_to_radar(point - pt.translation.xy());
                if b.length() < RADAR_HUD_OUTER_RADIUS - 0.1
                    && b.length() < RADAR_HUD_OUTER_RADIUS - 0.1
                {
                    let (color, alpha) = if traj.closest_flyby < 32.0 {
                        (
                            course_color_gradient
                                .0
                                .at(i as f64 / traj.path.len() as f64),
                            RADAR_PLANNED_COURSE_ALPHA,
                        )
                    } else {
                        (
                            course_color_gradient.0.at(250.0 / d as f64),
                            (RADAR_PLANNED_COURSE_ALPHA
                                * (1.0 - (i as f32 / traj.path.len() as f32)))
                                .powf(1.4),
                        )
                    };
                    painter.color =
                        Color::rgba(color.r as f32, color.g as f32, color.b as f32, alpha);
                    painter.thickness = ((i as f32 / max_segments).powf(3.0) * 15.0).max(0.5);
                    painter.line(a.extend(0.0), b.extend(0.0));
                }
            }
            last_point = Some(point);
        }
    }
}

pub fn world_to_radar(x: f32) -> f32 {
    ((x - RADAR_HUD_INNER_RADIUS) * RADAR_HUD_SCALE + RADAR_HUD_INNER_RADIUS)
        .clamp(RADAR_HUD_INNER_RADIUS, RADAR_HUD_OUTER_RADIUS)
}

pub fn vec_to_radar(v: Vec2) -> Vec2 {
    let (theta, r) = (v.y.atan2(v.x), v.length());
    let radar_r = world_to_radar(r);
    Vec2 {
        x: theta.cos() * radar_r,
        y: theta.sin() * radar_r,
    }
}

pub fn radar_arc_at_distance(object_radius: f32, distance: f32) -> f32 {
    (object_radius / distance).atan()
}

fn _radar_size_at_distance(object_radius: f32, distance: f32) -> f32 {
    object_radius / (distance * RADAR_HUD_SCALE)
}
