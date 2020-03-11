use crate::engine::bomb::{Bomb, BombId};
use crate::engine::explosion::{Explosion, ExplosionId};
use crate::engine::mob::{Mob, MobId};
use crate::engine::player::{Player, PlayerId};
use crate::engine::world::World;
use crate::tools::itemstore::ItemStore;
use crate::traits::celltypes::CellType;
use std::collections::HashMap;

pub type PlayerList = HashMap<PlayerId, Player>;
pub type MobList = HashMap<MobId, Mob>;
pub type ExplosionList = ItemStore<ExplosionId, Explosion>;
pub type BombList = ItemStore<BombId, Bomb>;

pub fn create_bomb_for_player(player: &mut Player, bombs: &mut BombList, world: &mut World) {
    if !player.has_bomb_remaining() {
        return;
    }

    let pos = player.position().to_map_position(world);
    match world.get_cell(pos) {
        Some(CellType::Empty) => {
            let bomb = Bomb::new(player, pos);
            add_bomb_to_world(bomb.clone(), bombs, world);
            player.bomb_placed();
            world.set_mob_data(pos, bomb.timestamp());
            update_bomb_path(bomb, world);
        }
        _ => {}
    }
}

pub fn update_bomb_path(bomb: Bomb, world: &mut World) {
    // TODO:
}

pub fn add_bomb_to_world(bomb: Bomb, bombs: &mut BombList, world: &mut World) {
    let pos = bomb.position();
    let id = bombs.add(bomb);
    world.add_bomb(id, pos);
}

pub fn add_explosion_to_world(
    explosion: Explosion,
    explosions: &mut ExplosionList,
    world: &mut World,
) {
    let pos = explosion.position();
    let id = explosions.add(explosion);
    world.add_explosion(id, pos);
}

pub fn process_explosions(
    delta_time: f64,
    explosions: &mut ExplosionList,
    bombs: &mut BombList,
    world: &mut World,
) {
    // Update remaining time for all bombs and explosions.
    for explosion in explosions.iter_mut() {
        explosion.update(delta_time);
    }

    explosions.retain(|_, e| e.is_active());

    let mut explode_new = Vec::new();
    for bomb in bombs.iter_mut() {
        if let Some(x) = bomb.tick(delta_time) {
            // Bomb exploded.
            explode_new.push(x);
        }
    }

    for explosion in explode_new.into_iter() {
        add_explosion_to_world(explosion, explosions, world);
    }

    bombs.retain(|_, b| b.is_active());
}
