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
            harmful: bomb.is_some(),
            timestamp: Timestamp::new(),
        }
    }

    pub fn id(&self) -> ExplosionId {
        self.id
    }

    pub fn pid(&self) -> PlayerId {
        self.pid
    }

    pub fn pname(&self) -> &str {
        &self.pname
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn is_harmful(&self) -> bool {
        self.harmful
    }

    pub fn position(&self) -> MapPosition {
        self.position
    }

    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }

    pub fn update(&mut self, delta_time: f64) {
        self.harmful = false;
        self.remaining -= delta_time;
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
            harmful: false,
            timestamp: Timestamp::new(),
        }
    }
}

impl From<(Bomb, MapPosition)> for Explosion {
    fn from(bomb: (Bomb, MapPosition)) -> Self {
        Explosion {
            id: ExplosionId::from(0),
            pid: bomb.0.pid(),
            pname: bomb.0.pname().to_string(),
            active: true,
            position: bomb.1,
            remaining: 0.5,
            harmful: true,
            timestamp: Timestamp::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        comms::playercomm::PlayerComm,
        engine::{player::{Player, PlayerId}, position::MapPosition},
    };
    use tokio::sync::mpsc;

    fn test_bomb_explosion() -> Explosion {
        let (tx, _rx_external) = mpsc::channel(1);
        let (_tx_external, rx) = mpsc::channel(1);
        let mut player = Player::new(PlayerId::from(1), PlayerComm::new(PlayerId::from(1), tx, rx));
        player.set_name("test");
        let bomb = Bomb::new(&player, MapPosition::new(1, 1), false);
        Explosion::from((bomb, MapPosition::new(1, 1)))
    }

    #[test]
    fn bomb_explosions_are_only_harmful_for_the_spawn_tick() {
        let mut explosion = test_bomb_explosion();

        assert!(explosion.is_harmful());
        assert!(explosion.is_active());

        explosion.update(1.0 / 30.0);

        assert!(!explosion.is_harmful());
        assert!(explosion.is_active());
    }
}
