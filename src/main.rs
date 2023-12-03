mod background;
mod camera;
mod player;

use bevy::{log::LogPlugin, prelude::*};
use bevy_rapier2d::plugin::{NoUserData, RapierConfiguration, RapierPhysicsPlugin, TimestepMode};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(LogPlugin {
            filter: "info,wgpu_core=error,wgpu_hal=error,space_chase=debug".into(),
            level: bevy::log::Level::DEBUG,
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(1.0))
        // .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, (camera::setup, player::setup, background::setup))
        .add_systems(
            Update,
            (player::control, camera::update, background::update).chain(),
        )
        .insert_resource(RapierConfiguration {
            gravity: Vec2::ZERO,
            timestep_mode: TimestepMode::Variable {
                max_dt: 1.0 / 60.0,
                time_scale: 1.0,
                substeps: 2,
            },
            ..default()
        })
        .run();
}
