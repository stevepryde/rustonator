use crate::{
    component::effect::Effect,
    engine::{
        bomb::{Bomb, BombId, BombRange, BombTime},
        explosion::{Explosion, ExplosionId},
        mob::{Mob, MobId},
        player::{Player, PlayerFlags, PlayerId},
        position::{MapPosition, PositionOffset},
        world::World,
    },
    error::ZResult,
    tools::itemstore::ItemStore,
    traits::celltypes::CellType,
};
use rand::Rng;
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
