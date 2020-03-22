use crate::{
    engine::{
        bomb::{Bomb, BombId},
        explosion::{Explosion, ExplosionId},
        mob::{Mob, MobId},
        player::{Player, PlayerId},
        position::{MapPosition, PositionOffset},
        world::World,
    },
    tools::itemstore::ItemStore,
    traits::celltypes::CellType,
};
use std::collections::HashMap;

pub type PlayerList = HashMap<PlayerId, Player>;
pub type MobList = HashMap<MobId, Mob>;
pub type ExplosionList = ItemStore<ExplosionId, Explosion>;
pub type BombList = ItemStore<BombId, Bomb>;

pub fn game_process_explosions(
    delta_time: f64,
    explosions: &mut ExplosionList,
    bombs: &mut BombList,
    world: &mut World,
)
{
    // Update remaining time for all bombs and explosions.
    for explosion in explosions.iter_mut() {
        explosion.update(delta_time);
        if !explosion.is_active() {
            world.clear_explosion_cell(explosion);
        }
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
        world.add_explosion(explosion, explosions);
    }

    bombs.retain(|_, b| b.is_active());
}

pub fn create_bomb_for_player(player: &mut Player, bombs: &mut BombList, world: &mut World) {
    if !player.has_bomb_remaining() {
        return;
    }

    let pos = player.position().to_map_position(world);
    if let Some(CellType::Empty) = world.get_cell(pos) {
        let bomb = Bomb::new(player, pos);
        player.bomb_placed();
        world.add_bomb(bomb, bombs);
    }
}
