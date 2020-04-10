use crate::{
    component::effect::Effect,
    engine::{
        bomb::{Bomb, BombId, BombRange, BombTime},
        explosion::{Explosion, ExplosionId},
        mob::{Mob, MobId},
        player::{Player, PlayerFlags, PlayerId},
        position::{MapPosition, PixelPositionF64, PositionOffset},
        world::World,
        worlddata::{InternalCellData, MobSpawner},
    },
    error::ZResult,
    tools::itemstore::ItemStore,
    traits::celltypes::CellType,
};
use rand::{seq::SliceRandom, Rng};
use std::collections::HashMap;

pub type PlayerList = HashMap<PlayerId, Player>;
pub type MobList = ItemStore<MobId, Mob>;
pub type ExplosionList = ItemStore<ExplosionId, Explosion>;
pub type BombList = ItemStore<BombId, Bomb>;

pub fn game_process_explosions_and_bombs(
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

pub async fn player_got_item(player: &mut Player, item: CellType) -> ZResult<bool> {
    match item {
        CellType::ItemBomb => {
            player.increase_max_bombs();
            player.ws().send_powerup("+B").await?;
            Ok(true)
        }
        CellType::ItemRange => {
            player.increase_range();
            player.ws().send_powerup("+R").await?;
            Ok(true)
        }
        CellType::ItemRandom => {
            let r: u8 = rand::thread_rng().gen_range(0, 10);
            let mut powerup_name = String::new();
            match r {
                0 => {
                    if player.max_bombs() < 6 {
                        player.increase_max_bombs();
                        powerup_name = "+B".to_owned();
                    }
                }
                1 => {
                    if player.max_bombs() > 1 {
                        player.decrease_max_bombs();
                        powerup_name = "-B".to_owned();
                    }
                }
                2 => {
                    if player.range() < BombRange::from(8) {
                        player.increase_range();
                        powerup_name = "+R".to_owned();
                    }
                }
                3 => {
                    if player.range() > BombRange::from(1) {
                        player.decrease_range();
                        powerup_name = "-R".to_owned();
                    }
                }
                4 => {
                    if player.has_flag(PlayerFlags::WALK_THROUGH_BOMBS) {
                        player.del_flag(PlayerFlags::WALK_THROUGH_BOMBS);
                        powerup_name = "-TB".to_owned();
                    } else {
                        player.add_flag(PlayerFlags::WALK_THROUGH_BOMBS);
                        powerup_name = "+TB".to_owned();
                    }
                }
                5 => {
                    if player.bomb_time() < BombTime::from(4.0) {
                        player.increase_bomb_time();
                        powerup_name = "SB".to_owned();
                    }
                }
                6 => {
                    if player.bomb_time() > BombTime::from(2.0) {
                        player.decrease_bomb_time();
                        powerup_name = "FB".to_owned();
                    }
                }
                7 => {
                    if player.score() > 100 {
                        let pwrup: u32 = rand::thread_rng().gen_range(1, 10) * 10;
                        player.decrease_score(pwrup);
                        powerup_name = "-$".to_owned();
                    }
                }
                8 => {
                    let pwrup: u32 = rand::thread_rng().gen_range(1, 10) * 10;
                    player.increase_score(pwrup);
                    powerup_name = "+$".to_owned();
                }
                _ => powerup_name = player.add_random_effect(),
            }

            if powerup_name.is_empty() {
                powerup_name = player.add_random_effect();
            }

            player.ws().send_powerup(&powerup_name).await?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

/// Spawn mob at a random mob spawner, and assign it a new target.
/// You'll want to pass in a clone of Vec<MobSpawner> because we shuffle it
/// in-place here.
pub fn spawn_mob(
    mobs: &mut MobList,
    mut spawners: Vec<MobSpawner>,
    players: &PlayerList,
    world: &World,
)
{
    let mob_positions: Vec<MapPosition> = mobs
        .iter()
        .map(|m| m.position().to_map_position(world))
        .collect();
    spawners.shuffle(&mut rand::thread_rng());
    for spawner in spawners {
        if !world.is_nearby_map_entity(spawner.position(), &mob_positions, 3) {
            let mut mob = Mob::new();
            mob.set_position(PixelPositionF64::from_map_position(
                spawner.position(),
                world,
            ));
            mob.choose_new_target(world, players);
            mobs.add(mob);
            break;
        }
    }
}

pub fn game_process_mobs(
    delta_time: f64,
    mobs: &mut MobList,
    players: &mut PlayerList,
    explosions: &ExplosionList,
    world: &World,
)
{
    for mob in mobs.iter_mut() {
        mob.update(delta_time, &players, &world);

        // Check if mob is dead.
        if let Some(InternalCellData::Explosion(explosion_id)) =
            world.get_internal_cell(mob.position().to_map_position(&world))
        {
            if let Some(explosion) = explosions.get(*explosion_id) {
                if explosion.is_active() {
                    mob.terminate();

                    // Award points to the player that killed this mob.
                    if let Some(p) = players.get_mut(&explosion.pid()) {
                        if !p.is_dead() {
                            if mob.is_smart() {
                                p.increase_score(2000);
                            } else {
                                p.increase_score(500);
                            }
                        }
                    }
                }
            }
        }
    }
}
