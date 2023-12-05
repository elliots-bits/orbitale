use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    prelude::*,
    render::{
        camera::CameraOutputMode,
        render_resource::{BlendState, LoadOp},
        view::{Layer, RenderLayers},
    },
};
use bevy_rapier2d::dynamics::Velocity;

use crate::player::PlayerMarker;

pub const GAME_LAYER: Layer = 0;
pub const UI_LAYER: Layer = 31;

#[derive(Component)]
pub struct GameCameraMarker;

#[derive(Component)]
pub struct UICameraMarker;

pub fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: 0,
                ..default()
            },
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::Custom(Color::Rgba {
                    red: 0.1,
                    green: 0.1,
                    blue: 0.15,
                    alpha: 1.0,
                }),
            },
            ..default()
        },
        RenderLayers::layer(GAME_LAYER),
        GameCameraMarker,
    ));
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: 1,
                output_mode: CameraOutputMode::Write {
                    blend_state: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    color_attachment_load_op: LoadOp::Load,
                },
                ..default()
            },
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::Custom(Color::hex("00000000").unwrap()),
            },
            ..default()
        },
        UICameraMarker,
        RenderLayers::layer(UI_LAYER),
    ));
}

pub fn update(
    mut camera: Query<
        (&mut Transform, &mut OrthographicProjection),
        (With<GameCameraMarker>, Without<PlayerMarker>),
    >,
    player: Query<(&Transform, &Velocity), With<PlayerMarker>>,
) {
    if let Ok((mut cam_t, mut proj)) = camera.get_single_mut() {
        if let Ok((player_t, player_v)) = player.get_single() {
            cam_t.translation = player_t.translation;

            let speed = player_v.linvel.length();
            let target_scale = (((speed / 600.0 - 1.0).tanh() + 1.0) / 2.0).powf(2.0) * 2.0 + 2.0;
            proj.scale = proj.scale + (target_scale - proj.scale) / 100.0;
        }
    }
}

pub fn game_layer() -> RenderLayers {
    RenderLayers::layer(GAME_LAYER)
}
