use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    prelude::*,
    render::{
        camera::CameraOutputMode,
        render_resource::{BlendState, LoadOp},
        view::{Layer, RenderLayers},
    },
};
use bevy_parallax::{
    CreateParallaxEvent, LayerData, LayerRepeat, LayerSpeed, ParallaxCameraComponent,
    ParallaxMoveEvent, ParallaxPlugin, ParallaxSystems, RepeatStrategy,
};
use bevy_rapier2d::dynamics::Velocity;

use crate::{player::PlayerMarker, AppState};

pub const GAME_LAYER: Layer = 0;
pub const UI_LAYER: Layer = 31;

#[derive(Component)]
pub struct GameCameraMarker;

#[derive(Component)]
pub struct UICameraMarker;

pub fn setup(app: &mut App) {
    app.add_plugins(ParallaxPlugin);
    app.add_systems(Startup, initialize_ui_camera);
    app.add_systems(OnEnter(AppState::Game), initialize_game_camera);
    app.add_systems(OnExit(AppState::Game), cleanup_game_camera);
    app.add_systems(
        Update,
        update_game_camera
            .before(ParallaxSystems)
            .run_if(in_state(AppState::Game)),
    );
}

fn initialize_game_camera(
    mut commands: Commands,
    mut create_parallax: EventWriter<CreateParallaxEvent>,
) {
    let camera = commands.spawn((
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
        ParallaxCameraComponent::default(),
    ));
    create_parallax.send(CreateParallaxEvent {
        layers_data: vec![
            LayerData {
                speed: LayerSpeed::Bidirectional(0.5, 0.5),
                repeat: LayerRepeat::Bidirectional(RepeatStrategy::Same, RepeatStrategy::Same),
                path: "stars_light.png".to_string(),
                tile_size: Vec2::new(1000.0, 1000.0),
                cols: 1,
                rows: 1,
                scale: 8.0,
                z: -1.0,
                ..Default::default()
            },
            LayerData {
                speed: LayerSpeed::Bidirectional(0.0, 0.0),
                repeat: LayerRepeat::Bidirectional(RepeatStrategy::Same, RepeatStrategy::Same),
                path: "stars_light.png".to_string(),
                tile_size: Vec2::new(1000.0, 1000.0),
                cols: 1,
                rows: 1,
                scale: 8.0,
                z: -1.0,
                ..Default::default()
            },
            LayerData {
                speed: LayerSpeed::Bidirectional(1.0, 1.0),
                repeat: LayerRepeat::Bidirectional(RepeatStrategy::Same, RepeatStrategy::Same),
                path: "nasa_milky_way.png".to_string(),
                tile_size: Vec2::new(9725.0, 4862.0),
                cols: 1,
                rows: 1,
                scale: 1.0,
                z: -2.0,
                ..Default::default()
            },
        ],
        camera: camera.id(),
    });
}

fn cleanup_game_camera(
    mut commands: Commands,
    query: Query<Entity, With<GameCameraMarker>>,
    parallax: Query<Entity, With<bevy_parallax::LayerComponent>>,
) {
    commands.entity(query.single()).despawn_recursive();
    for entity in parallax.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn initialize_ui_camera(mut commands: Commands) {
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

fn update_game_camera(
    mut camera: Query<
        (Entity, &Transform, &mut OrthographicProjection),
        (With<GameCameraMarker>, Without<PlayerMarker>),
    >,
    player: Query<(&Transform, &Velocity), With<PlayerMarker>>,
    mut move_event_writer: EventWriter<ParallaxMoveEvent>,
) {
    if let Ok((camera_entity, cam_t, mut proj)) = camera.get_single_mut() {
        if let Ok((player_t, player_v)) = player.get_single() {
            move_event_writer.send(ParallaxMoveEvent {
                camera_move_speed: (player_t.translation - cam_t.translation).xy(),
                camera: camera_entity,
            });

            // cam_t.translation = player_t.translation;

            let speed = player_v.linvel.length();
            let target_scale = (((speed / 600.0 - 1.0).tanh() + 1.0) / 2.0).powf(2.0) * 2.0 + 2.0;
            proj.scale = proj.scale + (target_scale - proj.scale) / 100.0;
        }
    }
}

pub fn game_layer() -> RenderLayers {
    RenderLayers::layer(GAME_LAYER)
}
