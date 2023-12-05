use bevy::prelude::*;
use bevy_rapier2d::pipeline::CollisionEvent;

use crate::{
    alien_ship::AlienShipMarker,
    despawn_queue::DespawnQueue,
    lasers::{Laser, LaserOrigin},
    player::PlayerMarker,
};

pub fn update(
    mut _commands: Commands,
    mut despawn_queue: ResMut<DespawnQueue>,
    mut collisions: EventReader<CollisionEvent>,
    player: Query<Entity, With<PlayerMarker>>,
    lasers: Query<(Entity, &Laser)>,
    alien_ships: Query<Entity, With<AlienShipMarker>>,
) {
    for event in collisions.read() {
        if let &CollisionEvent::Started(a, b, _) = event {
            debug!("Collision detected between {:?} and {:?}", a, b);

            // Check for an alien laser hitting the player
            if let Some((player, (laser_entity, laser))) =
                if let (Ok(p), Ok(l)) = (player.get(a), lasers.get(b)) {
                    Some((p, l))
                } else if let (Ok(p), Ok(l)) = (player.get(b), lasers.get(a)) {
                    Some((p, l))
                } else {
                    None
                }
            {
                if laser.origin == LaserOrigin::Enemy {
                    debug!("Player's been hit by enemy");
                    despawn_queue.1.insert(laser_entity);
                }
            }

            // Check for any laser hitting an alien ship
            if let Some((alien_ship, (laser_entity, _laser))) =
                if let (Ok(s), Ok(l)) = (alien_ships.get(a), lasers.get(b)) {
                    Some((s, l))
                } else if let (Ok(s), Ok(l)) = (alien_ships.get(b), lasers.get(a)) {
                    Some((s, l))
                } else {
                    None
                }
            {
                debug!("An alien ship has been hit");
                despawn_queue.1.insert(laser_entity);
                despawn_queue.1.insert(alien_ship);
            }
        }
    }
}
