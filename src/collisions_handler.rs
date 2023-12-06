use bevy::prelude::*;
use bevy_rapier2d::pipeline::CollisionEvent;

use crate::{
    alien_ship::AlienShipMarker,
    celestial_body::CelestialBodyMarker,
    despawn_queue::DespawnQueue,
    healthpoints::HealthPoints,
    lasers::{Laser, LaserOrigin},
    player::PlayerMarker,
};

pub fn update(
    mut _commands: Commands,
    mut despawn_queue: ResMut<DespawnQueue>,
    mut collisions: EventReader<CollisionEvent>,
    mut player: Query<&mut HealthPoints, (With<PlayerMarker>, Without<AlienShipMarker>)>,
    mut alien_ships: Query<&mut HealthPoints, (With<AlienShipMarker>, Without<PlayerMarker>)>,
    lasers: Query<(Entity, &Laser)>,
    celestial_bodies: Query<Entity, With<CelestialBodyMarker>>,
) {
    for event in collisions.read() {
        if let &CollisionEvent::Started(a, b, _) = event {
            // debug!("Collision detected between {:?} and {:?}", a, b);

            // Check for an alien laser hitting the player
            if let Some((mut player_hp, (laser_entity, laser))) =
                if let (Ok(p), Ok(l)) = (player.get_mut(a), lasers.get(b)) {
                    Some((p, l))
                } else if let (Ok(p), Ok(l)) = (player.get_mut(b), lasers.get(a)) {
                    Some((p, l))
                } else {
                    None
                }
            {
                if laser.origin == LaserOrigin::Enemy {
                    // debug!("Player's been hit by enemy");
                    despawn_queue.1.insert(laser_entity);
                    player_hp.decrease(laser.damage);
                }
            }

            // Check for any laser hitting an alien ship
            if let Some((mut alien_ship, (laser_entity, laser))) =
                if let (Ok(s), Ok(l)) = (alien_ships.get_mut(a), lasers.get(b)) {
                    Some((s, l))
                } else if let (Ok(s), Ok(l)) = (alien_ships.get_mut(b), lasers.get(a)) {
                    Some((s, l))
                } else {
                    None
                }
            {
                // debug!("An alien ship has been hit");
                despawn_queue.1.insert(laser_entity);
                alien_ship.decrease(laser.damage);
            }

            // Check for player ship hitting alien ship
            if let Some((mut alien_ship, _)) =
                if let (Ok(s), Ok(p)) = (alien_ships.get_mut(a), player.get(b)) {
                    Some((s, p))
                } else if let (Ok(s), Ok(p)) = (alien_ships.get_mut(b), player.get(a)) {
                    Some((s, p))
                } else {
                    None
                }
            {
                // debug!("The player has crashed into an alien ship");
                alien_ship.decrease(25.0);
            }

            // Check for player ship hitting celestial body
            if let Some((mut player_hp, _)) =
                if let (Ok(p), Ok(b)) = (player.get_mut(a), celestial_bodies.get(b)) {
                    Some((p, b))
                } else if let (Ok(p), Ok(b)) = (player.get_mut(b), celestial_bodies.get(a)) {
                    Some((p, b))
                } else {
                    None
                }
            {
                // debug!("The player has crashed into an alien ship");
                let hp = player_hp.current;
                player_hp.decrease(hp);
            }

            // Check for alien ship hitting celestial body
            if let Some((mut alien_hp, _)) =
                if let (Ok(p), Ok(b)) = (alien_ships.get_mut(a), celestial_bodies.get(b)) {
                    Some((p, b))
                } else if let (Ok(p), Ok(b)) = (alien_ships.get_mut(b), celestial_bodies.get(a)) {
                    Some((p, b))
                } else {
                    None
                }
            {
                // debug!("The player has crashed into an alien ship");
                let hp = alien_hp.current;
                alien_hp.decrease(hp);
            }

            // Check for laser hitting celestial body
            // Check for any laser hitting an alien ship
            if let Some((_, (laser_entity, _))) =
                if let (Ok(s), Ok(l)) = (celestial_bodies.get(a), lasers.get(b)) {
                    Some((s, l))
                } else if let (Ok(s), Ok(l)) = (celestial_bodies.get(b), lasers.get(a)) {
                    Some((s, l))
                } else {
                    None
                }
            {
                // debug!("An alien ship has been hit");
                despawn_queue.1.insert(laser_entity);
            }

            // // Check for alien ship hitting alien ship
            // if let (Ok(a1), Ok(a2)) = (alien_ships.get(a), alien_ships.get(b)) {
            //     // debug!("Two alien ships crashed into each other");
            //     despawn_queue.1.insert(a1);
            //     despawn_queue.1.insert(a2);
            // }
        }
    }
}
