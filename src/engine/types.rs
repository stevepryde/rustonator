use crate::{
    engine::{
        bomb::{Bomb, BombId},
        explosion::{Explosion, ExplosionId},
        mob::{Mob, MobId},
        player::{Player, PlayerId},
    },
    tools::itemstore::ItemStore,
};
use std::collections::HashMap;

pub type PlayerList = HashMap<PlayerId, Player>;
pub type MobList = ItemStore<MobId, Mob>;
pub type ExplosionList = ItemStore<ExplosionId, Explosion>;
pub type BombList = ItemStore<BombId, Bomb>;
