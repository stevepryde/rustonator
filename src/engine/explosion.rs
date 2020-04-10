use crate::{
    engine::{bomb::Bomb, player::PlayerId, position::MapPosition},
    tools::itemstore::HasId,
    utils::misc::Timestamp,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ExplosionId(u64);

impl From<u64> for ExplosionId {
    fn from(value: u64) -> Self {
        ExplosionId(value)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Explosion {
    id: ExplosionId,
    pid: PlayerId,
    pname: String,
    active: bool,
    #[serde(flatten)]
    position: MapPosition,
    remaining: f64,
    harmful: bool,
    timestamp: Timestamp,
}

impl Explosion {
    pub fn new(bomb: Option<&Bomb>, position: MapPosition) -> Self {
        Explosion {
            id: ExplosionId::from(0),
            pid: bomb.map_or(PlayerId::from(0), |x| x.pid()),
            pname: bomb.map_or(String::new(), |x| x.pname().to_owned()),
            active: true,
            position,
            remaining: 0.5,
            harmful: true,
            timestamp: Timestamp::new(),
        }
    }

    pub fn id(&self) -> ExplosionId {
        self.id
    }

    pub fn pid(&self) -> PlayerId {
        self.pid
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn position(&self) -> MapPosition {
        self.position
    }

    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }

    pub fn update(&mut self, delta_time: f64) {
        self.remaining -= delta_time;
        if self.remaining <= 0.3 {
            self.harmful = false;
        }

        if self.remaining <= 0.0 {
            self.remaining = 0.0;
            self.active = false;
        }
    }
}

impl HasId<ExplosionId> for Explosion {
    fn set_id(&mut self, id: ExplosionId) {
        self.id = id;
    }
}

impl From<MapPosition> for Explosion {
    fn from(position: MapPosition) -> Self {
        Explosion {
            id: ExplosionId::from(0),
            pid: PlayerId::from(0),
            pname: String::new(),
            active: true,
            position,
            remaining: 0.5,
            harmful: true,
            timestamp: Timestamp::new(),
        }
    }
}

impl From<Bomb> for Explosion {
    fn from(bomb: Bomb) -> Self {
        Explosion {
            id: ExplosionId::from(0),
            pid: bomb.pid(),
            pname: bomb.pname().to_string(),
            active: true,
            position: bomb.position(),
            remaining: 0.5,
            harmful: true,
            timestamp: Timestamp::new(),
        }
    }
}
