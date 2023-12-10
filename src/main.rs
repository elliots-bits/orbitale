mod ai;
mod alien_ship;
mod alien_waves;
mod camera;
mod celestial_body;
mod collisions_handler;
mod course_planner;
mod death;
mod despawn_queue;
mod gravity;
mod healthpoints;
mod impulses_aggregator;
mod lasers;
mod particles;
mod player;
mod system_sets;
mod thruster;
mod ui;

use bevy::{
    a11y::AccessibilityPlugin, audio::AudioPlugin, core_pipeline::CorePipelinePlugin,
    diagnostic::DiagnosticsPlugin, input::InputPlugin, log::LogPlugin, prelude::*,
    render::RenderPlugin, scene::ScenePlugin, sprite::SpritePlugin, text::TextPlugin,
    time::TimePlugin, ui::UiPlugin, winit::WinitPlugin,
};
use bevy_parallax::ParallaxPlugin;
use bevy_rapier2d::plugin::{NoUserData, RapierConfiguration, RapierPhysicsPlugin, TimestepMode};
use bevy_vector_shapes::Shape2dPlugin;
use system_sets::AppStage;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    Game,
    #[default]
    Menu,
    DeathScreen,
}

fn main() {
    let mut app = App::new();

    app.add_state::<AppState>();
    app.add_plugins(LogPlugin {
        filter: "info,wgpu_core=error,wgpu_hal=error,space_chase=debug".into(),
        level: bevy::log::Level::DEBUG,
    });
    app.add_plugins(TaskPoolPlugin::default());
    app.add_plugins(TypeRegistrationPlugin);
    app.add_plugins(FrameCountPlugin);
    app.add_plugins(TimePlugin);
    app.add_plugins(TransformPlugin);
    app.add_plugins(HierarchyPlugin);
    app.add_plugins(DiagnosticsPlugin);
    app.add_plugins(InputPlugin);
    app.add_plugins(WindowPlugin::default());
    app.add_plugins(AccessibilityPlugin);
    app.add_plugins(AssetPlugin::default());
    app.add_plugins(ScenePlugin);
    app.add_plugins(WinitPlugin::default());
    app.add_plugins(RenderPlugin::default());
    app.add_plugins(ImagePlugin::default());
    app.add_plugins(CorePipelinePlugin);
    app.add_plugins(SpritePlugin);
    app.add_plugins(TextPlugin);
    app.add_plugins(UiPlugin);
    app.add_plugins(AudioPlugin::default());
    app.add_plugins(GilrsPlugin);
    app.add_plugins(AnimationPlugin);
    app.add_plugins(ParallaxPlugin);
    app.add_plugins(Shape2dPlugin::default());
    app.add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(1.0));

    camera::setup(&mut app);
    impulses_aggregator::setup(&mut app);
    despawn_queue::setup(&mut app);
    system_sets::setup(&mut app);
    ui::setup(&mut app);

    app.insert_resource(RapierConfiguration {
        gravity: Vec2::ZERO,
        timestep_mode: TimestepMode::Variable {
            max_dt: 1.0 / 20.0,
            time_scale: 1.0,
            substeps: 2,
        },
        ..default()
    });
    app.add_systems(
        OnExit(AppState::Game),
        (
            player::cleanup,
            celestial_body::cleanup,
            alien_ship::cleanup,
        )
            .chain(),
    );

    app.add_systems(
        OnEnter(AppState::Game),
        (
            player::setup,
            alien_waves::setup,
            celestial_body::setup,
            ai::setup,
        )
            .chain(),
    );

    app.add_systems(
        Update,
        (player::control, alien_waves::update, alien_ship::update)
            .in_set(AppStage::Control)
            .run_if(in_state(AppState::Game)),
    );

    app.add_systems(
        Update,
        (ai::update_ai_states, ai::update_ai_controllers)
            .chain()
            .in_set(AppStage::AI)
            .run_if(in_state(AppState::Game)),
    );

    app.add_systems(
        Update,
        (
            course_planner::compute_player_trajectory,
            course_planner::compute_enemies_trajectories,
        )
            .in_set(AppStage::Trajectories)
            .run_if(in_state(AppState::Game)),
    );
    app.add_systems(
        Update,
        (
            thruster::update,
            lasers::update,
            collisions_handler::update,
            gravity::update,
            celestial_body::update,
            death::update,
            particles::update,
            particles::thrusters::spawn_main_thruster_particles,
        )
            .in_set(AppStage::Simulation)
            .run_if(in_state(AppState::Game)),
    );
    app.add_systems(
        Update,
        (lasers::draw, particles::draw)
            .in_set(AppStage::Draw)
            .run_if(in_state(AppState::Game)),
    );
    app.run();
}
