use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier2d::{dynamics::Velocity, pipeline::CollisionEvent};

use crate::{
    alien_ship::AlienShipMarker,
    celestial_body::CelestialBodyMarker,
    despawn_queue::DespawnQueue,
    gravity::AffectedByGravity,
    healthpoints::HealthPoints,
    impulses_aggregator::AddExternalImpulse,
    lasers::{Laser, LaserOrigin},
    player::{PlayerMarker, PLAYER_MASS},
    thruster::Thruster,
    ui::{Difficulty, GameSettings},
    GLOBAL_IMPULSE_DURATION_MULT,
};

const LASER_HIT_LINEAR_IMPULSE: f32 = 1.0 * GLOBAL_IMPULSE_DURATION_MULT;
const LASER_HIT_ANGULAR_IMPULSE: f32 = 150.0 * GLOBAL_IMPULSE_DURATION_MULT;

pub fn update(
    mut despawn_queue: ResMut<DespawnQueue>,
    mut collisions: EventReader<CollisionEvent>,
    mut player: Query<
        (Entity, &Transform, &Velocity, &mut HealthPoints),
        (With<PlayerMarker>, Without<AlienShipMarker>),
    >,
    mut alien_ships: Query<
        (Entity, &mut HealthPoints, &Transform, &Velocity),
        (With<AlienShipMarker>, Without<PlayerMarker>),
    >,
    mut lasers: Query<(Entity, &Transform, &Velocity, &mut Laser)>,
    celestial_bodies: Query<Entity, With<CelestialBodyMarker>>,
    mut impulses: EventWriter<AddExternalImpulse>,
    player_movement_query: Query<(&AffectedByGravity, &Thruster, &Transform)>,
    settings: Res<GameSettings>,
) {
    for event in collisions.read() {
        if let &CollisionEvent::Started(a, b, _contact) = event {
            // debug!("Collision detected between {:?} and {:?}", a, b);

            // Check for an alien laser hitting the player
            if let Some(((e, pt, pv, mut player_hp), (laser_entity, lt, lv, mut laser))) =
                if let (Ok(p), Ok(l)) = (player.get_mut(a), lasers.get_mut(b)) {
                    Some((p, l))
                } else if let (Ok(p), Ok(l)) = (player.get_mut(b), lasers.get_mut(a)) {
                    Some((p, l))
                } else {
                    None
                }
            {
                if laser.origin == LaserOrigin::Enemy {
                    // debug!("Player's been hit by enemy");
                    despawn_queue.1.insert(laser_entity);
                    player_hp.decrease(laser.damage, settings.difficulty);
                    laser.damage = 0.0;

                    // Compute linear and angular knockback
                    let dv = pv.linvel - lv.linvel;
                    let dp = pt.translation.xy() - lt.translation.xy();
                    let dot = -dv.normalize_or_zero().dot(dp.normalize_or_zero());
                    let perp_dot = -dv.normalize_or_zero().perp_dot(dp.normalize_or_zero());
                    let linear =
                        dp.normalize_or_zero() * LASER_HIT_LINEAR_IMPULSE * dot * PLAYER_MASS;
                    let angular = LASER_HIT_ANGULAR_IMPULSE * perp_dot * PLAYER_MASS;
                    impulses.send(AddExternalImpulse {
                        entity: e,
                        impulse: linear,
                        torque_impulse: angular,
                    })
                }
            }

            // Check for any laser hitting an alien ship
            if let Some(((e, mut alien_ship, at, av), (laser_entity, lt, lv, laser))) =
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
                alien_ship.decrease(laser.damage, Difficulty::Normal);

                // Compute linear and angular knockback
                let dv = av.linvel - lv.linvel;
                let dp = at.translation.xy() - lt.translation.xy();
                let dot = -dv.normalize_or_zero().dot(dp.normalize_or_zero());
                let perp_dot = -dv.normalize_or_zero().perp_dot(dp.normalize());
                let linear = dp.normalize_or_zero() * LASER_HIT_LINEAR_IMPULSE * dot;
                let angular = LASER_HIT_ANGULAR_IMPULSE * perp_dot;
                impulses.send(AddExternalImpulse {
                    entity: e,
                    impulse: linear,
                    torque_impulse: angular,
                })
            }

            // Check for player ship hitting alien ship
            if let Some(((_e, mut alien_ship, _, _), _)) =
                if let (Ok(s), Ok(p)) = (alien_ships.get_mut(a), player.get(b)) {
                    Some((s, p))
                } else if let (Ok(s), Ok(p)) = (alien_ships.get_mut(b), player.get(a)) {
                    Some((s, p))
                } else {
                    None
                }
            {
                // debug!("The player has crashed into an alien ship");
                alien_ship.decrease(25.0, Difficulty::Normal);
            }

            // Check for player ship hitting celestial body
            if let Some(((player_entity, _pt, _pv, mut player_hp), _)) =
                if let (Ok(p), Ok(b)) = (player.get_mut(a), celestial_bodies.get(b)) {
                    Some((p, b))
                } else if let (Ok(p), Ok(b)) = (player.get_mut(b), celestial_bodies.get(a)) {
                    Some((p, b))
                } else {
                    None
                }
            {
                // debug!("The player has crashed into an alien ship");
                let hp = player_hp.max;
                player_hp.decrease(hp, settings.difficulty);

                if settings.difficulty == Difficulty::GodMode {
                    let (gravity, _thruster, _transform) =
                        player_movement_query.get(player_entity).unwrap();

                    impulses.send(AddExternalImpulse {
                        entity: player_entity,
                        impulse: (gravity.last_acceleration * -0.4)
                            .rotate(Vec2::from_angle(PI / 4.)),
                        torque_impulse: 0.,
                    });
                }
            }

            // Check for alien ship hitting celestial body
            if let Some(((_e, mut alien_hp, _, _), _)) =
                if let (Ok(p), Ok(b)) = (alien_ships.get_mut(a), celestial_bodies.get(b)) {
                    Some((p, b))
                } else if let (Ok(p), Ok(b)) = (alien_ships.get_mut(b), celestial_bodies.get(a)) {
                    Some((p, b))
                } else {
                    None
                }
            {
                // debug!("The player has crashed into an alien ship");
                let hp = alien_hp.max;
                alien_hp.decrease(hp, Difficulty::Normal);
            }

            // Check for laser hitting celestial body
            if let Some((_, (laser_entity, _, _, _))) =
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
