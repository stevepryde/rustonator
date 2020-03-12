use crate::engine::player::PlayerId;
use crate::utils::misc::Timestamp;
use crate::{
    engine::{explosion::Explosion, player::Player, position::MapPosition},
    tools::itemstore::HasId,
};
use bitflags::_core::ops::Deref;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BombId(u64);

impl From<u64> for BombId {
    fn from(value: u64) -> Self {
        BombId(value)
    }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct BombRange(u32);

impl From<u32> for BombRange {
    fn from(value: u32) -> Self {
        BombRange(value)
    }
}

impl Deref for BombRange {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Bomb {
    id: BombId,
    pid: PlayerId,
    pname: String,
    active: bool,
    #[serde(flatten)]
    position: MapPosition,
    remaining: f64, // TODO: wrap this in a newtype TimeRemaining(f64)
    range: BombRange,
    timestamp: Timestamp,
}

impl Bomb {
    pub fn new(player: &Player, position: MapPosition) -> Self {
        Bomb {
            id: BombId::from(0),
            pid: player.id(),
            pname: player.name().to_owned(),
            active: true,
            position,
            remaining: player.bomb_time(),
            range: player.range(),
            timestamp: Timestamp::new(),
        }
    }

    pub fn id(&self) -> BombId {
        self.id
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn pid(&self) -> PlayerId {
        self.pid
    }

    pub fn pname(&self) -> &str {
        self.pname.as_str()
    }

    pub fn position(&self) -> MapPosition {
        self.position
    }

    pub fn range(&self) -> BombRange {
        self.range
    }

    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }

    pub fn tick(&mut self, delta_time: f64) -> Option<Explosion> {
        self.remaining -= delta_time;
        if self.remaining <= 0.0 {
            self.remaining = 0.0;
            self.active = false;
            Some(Explosion::new(Some(&self), self.position))
        } else {
            None
        }
    }

    pub fn terminate(&mut self) {
        self.active = false;
    }
}

impl HasId<BombId> for Bomb {
    fn set_id(&mut self, id: BombId) {
        self.id = id;
    }
}
