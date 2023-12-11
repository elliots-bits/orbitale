use bevy::prelude::*;

use crate::AppState;

const PRIMARY_COLOR: Color = Color::rgb(0.95, 0.95, 0.95);

#[derive(Component)]
pub struct CreditsToMenuButton;

#[derive(Component)]
pub struct CreditsScreen;

pub fn setup(app: &mut App) {
    app.add_systems(OnEnter(AppState::Credits), setup_credits_screen);
    app.add_systems(OnExit(AppState::Credits), cleanup_credits_screen);
    app.add_systems(
        Update,
        (update_credits_screen, menu_on_press_space).run_if(in_state(AppState::Credits)),
    );
}

fn setup_credits_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    let credits = commands
        .spawn((
            NodeBundle {
                style: Style {
                    // center button
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: Color::WHITE.into(),
                ..default()
            },
            UiImage::new(asset_server.load("nasa_milky_way.webp")),
            CreditsScreen,
        ))
        .id();

    let artists = commands
        .spawn(
            TextBundle::from_section(
                "Created by Elliot Bitsch & Eliott Gaboreau",
                TextStyle {
                    font_size: 50.0,
                    font: asset_server.load("fusion-pixel-12px-proportional-latin.ttf"),
                    color: PRIMARY_COLOR,
                },
            )
            .with_style(Style {
                margin: UiRect {
                    bottom: Val::Px(20.),
                    ..Default::default()
                },
                ..Default::default()
            }),
        )
        .id();
    commands.entity(credits).add_child(artists);

    let engine: Entity = commands
        .spawn(
            TextBundle::from_section(
                "Implemented using the Bevy game engine",
                TextStyle {
                    font_size: 30.0,
                    font: asset_server.load("fusion-pixel-12px-proportional-latin.ttf"),
                    color: PRIMARY_COLOR,
                },
            )
            .with_style(Style {
                margin: UiRect {
                    bottom: Val::Px(16.),
                    ..Default::default()
                },
                ..Default::default()
            }),
        )
        .id();
    commands.entity(credits).add_child(engine);

    let physics: Entity = commands
        .spawn(
            TextBundle::from_section(
                "Collisions and forces computed by the Rapier engine, glued to bevy using bevy-rapier",
                TextStyle {
                    font_size: 30.0,
                    font: asset_server.load("fusion-pixel-12px-proportional-latin.ttf"),
                    color: PRIMARY_COLOR,
                },
            )
            .with_style(Style {
                margin: UiRect {
                    bottom: Val::Px(16.),
                    ..Default::default()
                },
                ..Default::default()
            }),
        )
        .id();
    commands.entity(credits).add_child(physics);

    let parallax: Entity = commands
        .spawn(
            TextBundle::from_section(
                "Parallax background effect using the bevy-parallax crate",
                TextStyle {
                    font_size: 30.0,
                    font: asset_server.load("fusion-pixel-12px-proportional-latin.ttf"),
                    color: PRIMARY_COLOR,
                },
            )
            .with_style(Style {
                margin: UiRect {
                    bottom: Val::Px(16.),
                    ..Default::default()
                },
                ..Default::default()
            }),
        )
        .id();
    commands.entity(credits).add_child(parallax);

    let vector_shapes: Entity = commands
        .spawn(
            TextBundle::from_section(
                "Particles and HUD rendered with bevy_vector_shapes",
                TextStyle {
                    font_size: 30.0,
                    font: asset_server.load("fusion-pixel-12px-proportional-latin.ttf"),
                    color: PRIMARY_COLOR,
                },
            )
            .with_style(Style {
                margin: UiRect {
                    bottom: Val::Px(16.),
                    ..Default::default()
                },
                ..Default::default()
            }),
        )
        .id();
    commands.entity(credits).add_child(vector_shapes);

    let wasm: Entity = commands
        .spawn(
            TextBundle::from_section(
                "Built for web browsers thanks to the amazing work of the wasm-bindgen community",
                TextStyle {
                    font_size: 30.0,
                    font: asset_server.load("fusion-pixel-12px-proportional-latin.ttf"),
                    color: PRIMARY_COLOR,
                },
            )
            .with_style(Style {
                margin: UiRect {
                    bottom: Val::Px(16.),
                    ..Default::default()
                },
                ..Default::default()
            }),
        )
        .id();
    commands.entity(credits).add_child(wasm);

    let nasa: Entity = commands
        .spawn(
            TextBundle::from_section(
                "Background image: The center of our galaxy captured by 3 telescopes: NASA/JPL-Caltech/ESA/CXC/STScI",
                
                TextStyle {
                    font_size: 30.0,
                    font: asset_server.load("fusion-pixel-12px-proportional-latin.ttf"),
                    color: PRIMARY_COLOR,
                },
            )
            .with_style(Style {
                margin: UiRect {
                    bottom: Val::Px(16.),
                    ..Default::default()
                },
                ..Default::default()
            }),
        )
        .id();
    commands.entity(credits).add_child(nasa);

    let menu_button = commands
        .spawn((
            ButtonBundle {
                style: Style {
                    // horizontally center child text
                    justify_content: JustifyContent::Center,
                    // vertically center child text
                    align_items: AlignItems::Center,
                    padding: UiRect {
                        left: Val::Px(20.),
                        right: Val::Px(20.),
                        top: Val::Px(0.0),
                        bottom: Val::Px(7.),
                    },
                    border: UiRect {
                        left: Val::Px(2.),
                        right: Val::Px(2.),
                        top: Val::Px(2.),
                        bottom: Val::Px(2.),
                    },
                    top: Val::Px(5.0),
                    margin: UiRect {
                        left: Val::Px(0.),
                        right: Val::Px(0.),
                        top: Val::Px(24.),
                        bottom: Val::Px(0.),
                    },
                    ..default()
                },
                background_color: Color::NONE.into(),
                border_color: PRIMARY_COLOR.into(),
                ..default()
            },
            CreditsToMenuButton,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Menu",
                TextStyle {
                    font_size: 40.0,
                    color: PRIMARY_COLOR,
                    font: asset_server.load("fusion-pixel-12px-proportional-latin.ttf"),
                },
            ));
        })
        .id();
    commands.entity(credits).add_child(menu_button);
}

fn update_credits_screen(
    mut next_state: ResMut<NextState<AppState>>,
    mut menu_button_interraction: Query<
        (&Interaction, &Children, &mut BackgroundColor),
        (
            Changed<Interaction>,
            With<Button>,
            With<CreditsToMenuButton>,
        ),
    >,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, children, mut background_color) in &mut menu_button_interraction {
        let mut text = text_query.get_mut(children[0]).unwrap();

        match *interaction {
            Interaction::Pressed => {
                next_state.set(AppState::Menu);
            }
            Interaction::Hovered => {
                text.sections[0].style.color = Color::BLACK;
                background_color.0 = PRIMARY_COLOR;
            }
            Interaction::None => {
                text.sections[0].style.color = PRIMARY_COLOR;
                background_color.0 = Color::NONE;
            }
        }
    }
}

fn menu_on_press_space(mut next_state: ResMut<NextState<AppState>>, keys: Res<Input<KeyCode>>) {
    if keys.pressed(KeyCode::Space) {
        next_state.set(AppState::Menu);
    }
}

fn cleanup_credits_screen(
    mut commands: Commands,
    credits_screen_query: Query<Entity, With<CreditsScreen>>,
    camera_query: Query<Entity, With<Camera>>,
) {
    commands
        .entity(credits_screen_query.single())
        .despawn_recursive();
    commands.entity(camera_query.single()).despawn_recursive();
}
