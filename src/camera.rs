use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*};
use bevy_rapier2d::dynamics::Velocity;

use crate::player::PlayerMarker;

#[derive(Component)]
pub struct GameCameraMarker;

pub fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_xyz(100.0, 200.0, 0.0),
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
        GameCameraMarker,
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
            let speed = player_v.linvel.length();
            let scale = (((speed / 600.0 - 1.0).tanh() + 1.0) / 2.0).powf(2.0) * 3.0 + 1.0;
            cam_t.translation = player_t.translation;
            proj.scale = scale;
        }
    }
}
