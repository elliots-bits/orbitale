use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use crate::camera::GameCameraMarker;

#[derive(Component)]
pub struct BackgroundTileMarker;

#[derive(Resource, Default)]
pub struct ParralaxBackground {
    pub tiles: HashMap<(i32, i32), Vec<Entity>>,
    pub tilesize: (u32, u32),
    pub texture: Handle<Image>,
}

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(ParralaxBackground {
        texture: asset_server.load("stars_parallax.png"),
        tilesize: (1000, 1000),
        ..default()
    });
}

pub fn update(
    mut commands: Commands,
    mut background: ResMut<ParralaxBackground>,
    camera: Query<(&Transform, &OrthographicProjection), With<GameCameraMarker>>,
    tiles: Query<Entity, With<BackgroundTileMarker>>,
) {
    if let Ok((cam_transform, cam_proj)) = camera.get_single() {
        let cam_pos = cam_transform.translation.xy();
        // debug!("Cam pos: {:?}", cam_pos);
        let cam_area = cam_proj.area;
        // debug!("Cam area: {:?}", cam_area);
        let current_tile_min_index = (
            ((cam_area.min.x + cam_pos.x) / background.tilesize.0 as f32).floor() as i32,
            ((cam_area.min.y + cam_pos.y) / background.tilesize.1 as f32).floor() as i32,
        );
        let current_tile_max_index = (
            ((cam_area.max.x + cam_pos.x) / background.tilesize.0 as f32).ceil() as i32,
            ((cam_area.max.y + cam_pos.y) / background.tilesize.1 as f32).ceil() as i32,
        );
        let mut visible_stacks_indices = HashSet::<(i32, i32)>::new();
        // debug!("{:?} {:?}", current_tile_min_index, current_tile_max_index);
        for x in (current_tile_min_index.0 - 1)..(current_tile_max_index.0 + 1) {
            for y in (current_tile_min_index.1 - 1)..(current_tile_max_index.1 + 1) {
                visible_stacks_indices.insert((x, y));
            }
        }
        let mut loaded_stacks_indices = HashSet::<(i32, i32)>::new();
        for &(x, y) in background.tiles.keys() {
            loaded_stacks_indices.insert((x, y));
        }
        for stack_index_to_unload in loaded_stacks_indices.difference(&visible_stacks_indices) {
            // debug!("Unloading background at {:?}", stack_index_to_unload);
            if let Some(entities_to_despawn) = background.tiles.remove(stack_index_to_unload) {
                for e in entities_to_despawn {
                    commands.entity(e).despawn_recursive();
                }
            } else {
                panic!("This algorithm is very wrong.");
            }
        }
        for &stack_index_to_load in visible_stacks_indices.difference(&loaded_stacks_indices) {
            // debug!("Loading background at {:?}", stack_index_to_load);
            let ecmd = commands.spawn(SpriteBundle {
                texture: background.texture.clone(),
                transform: Transform::from_translation(Vec3 {
                    x: (stack_index_to_load.0 * background.tilesize.0 as i32) as f32,
                    y: (stack_index_to_load.1 * background.tilesize.1 as i32) as f32,
                    z: -1.0,
                }),
                ..default()
            });
            let ecmd = commands.spawn(SpriteBundle {
                texture: background.texture.clone(),
                transform: Transform::from_translation(Vec3 {
                    x: (stack_index_to_load.0 * background.tilesize.0 as i32) as f32,
                    y: (stack_index_to_load.1 * background.tilesize.1 as i32) as f32,
                    z: -1.0,
                }),
                ..default()
            });
            background
                .tiles
                .insert(stack_index_to_load, vec![ecmd.id()]); // todo: Build a true parralax stack
        }
    } else {
        panic!("Can't update background: camera not found.");
    }
}
