use crate::engine::player::PlayerId;
use crate::utils::misc::Timestamp;
use crate::{
    engine::{explosion::Explosion, player::Player, position::MapPosition},
    tools::itemstore::HasId,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BombId(u64);

impl From<u64> for BombId {
    fn from(value: u64) -> Self {
        BombId(value)
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
    range: u32,     // TODO: wrap this in a newtype BombRange(u32)
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
}

impl HasId<BombId> for Bomb {
    fn set_id(&mut self, id: BombId) {
        self.id = id;
    }
}
