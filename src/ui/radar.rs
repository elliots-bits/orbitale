use std::f32::consts::PI;

use bevy::{prelude::*, render::view::RenderLayers};
use bevy_rapier2d::{dynamics::Velocity, geometry::Collider};
use bevy_vector_shapes::{
    painter::ShapePainter,
    shapes::{Cap, DiscPainter, LinePainter, RectPainter},
};
use colorgrad::CustomGradient;

use crate::{
    alien_ship::AlienShipMarker,
    camera::UI_LAYER,
    celestial_body::CelestialBodyMarker,
    course_planner::{ComputedTrajectory, PLAYER_PLAN_DURATION, PLAYER_PLAN_STEP_DT},
    player::PlayerMarker,
    AppState,
};

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

pub fn setup(app: &mut App) {
    app.add_systems(OnEnter(AppState::Game), setup_radar_hud);

    // app.add_systems(
    //     Update,

    //         .in_set(AppStage::Draw)
    //         .run_if(in_state(AppState::Game)),
    // );
}

pub fn setup_radar_hud(mut commands: Commands) {
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

pub fn draw_radar_hud(
    mut painter: ShapePainter,
    ship_color_gradient: Res<RadarShipsColorGradient>,
    course_color_gradient: Res<CoursePlanningColorGradient>,
    player: Query<(&Transform, &Velocity, &ComputedTrajectory), With<PlayerMarker>>,
    alien_ships: Query<(&Transform, &Velocity), With<AlienShipMarker>>,
    celestial_bodies: Query<(&Transform, &Collider), With<CelestialBodyMarker>>,
) {
    if let Ok((pt, pv, traj)) = player.get_single() {
        // Draw Radar circles
        painter.reset();
        painter.render_layers = Some(RenderLayers::layer(UI_LAYER));
        painter.set_translation(Vec3::ZERO);
        painter.set_rotation(Quat::from_axis_angle(Vec3::ONE, 0.003).normalize());
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
        let start = Vec2::new(theta.cos() * 32.0, theta.sin() * 32.0);
        let end = Vec2::new(
            theta.cos() * RADAR_HUD_INNER_RADIUS,
            theta.sin() * RADAR_HUD_INNER_RADIUS,
        );
        painter.set_rotation(Quat::from_axis_angle(Vec3::Z, theta));
        painter.set_translation(((start + end) / 2.0).extend(0.0));
        painter.rect(Vec2::new(RADAR_HUD_INNER_RADIUS - 32.0, 1.0));

        for (at, av) in alien_ships.iter() {
            let dp = at.translation.xy() - pt.translation.xy();
            let dv = av.linvel - pv.linvel;

            let (theta, r) = (dp.y.atan2(dp.x), dp.length());
            let radar_r = world_to_radar(r);

            let ship_closing_speed =
                dv.length() * dv.normalize_or_zero().dot(dp.normalize_or_zero());

            let speed_color_interp = 1.0
                - ((ship_closing_speed + RADAR_COLOR_MAX_SPEED) / (RADAR_COLOR_MAX_SPEED * 2.0))
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
                painter.set_rotation(Quat::default());
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

        // Draw planned course
        let max_segments = PLAYER_PLAN_DURATION / PLAYER_PLAN_STEP_DT;

        painter.set_rotation(Quat::default());
        for (i, &(point, d)) in traj.path.iter().enumerate() {
            let p = vec_to_radar(point - pt.translation.xy());
            if p.length() < RADAR_HUD_OUTER_RADIUS - 0.1 {
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
                        (RADAR_PLANNED_COURSE_ALPHA * (1.0 - (i as f32 / traj.path.len() as f32)))
                            .powf(1.4),
                    )
                };
                painter.set_translation(p.extend(0.0));
                painter.hollow = true;
                painter.color = Color::rgba(color.r as f32, color.g as f32, color.b as f32, alpha);
                painter.thickness = ((i as f32 / max_segments).powf(3.0) * 15.0).max(0.5);
                painter.circle(0.5);
            }
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
