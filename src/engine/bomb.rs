use crate::{
    engine::{explosion::Explosion, player::Player, position::MapPosition},
    tools::itemstore::HasId,
    traits::worldobject::{JsonError, ToJson},
    utils::misc::unix_timestamp,
};
use serde::{Deserialize, Serialize};
use serde_json;
use std::convert::TryFrom;

// TODO: use type system for ids for better type safety.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bomb {
    id: u32,
    pid: String,
    pname: String,
    active: bool,
    #[serde(flatten)]
    position: MapPosition,
    remaining: f32,
    range: u32,
    timestamp: i64,
}

impl Bomb {
    pub fn new(player: &Player, position: MapPosition) -> Self {
        Bomb {
            id: 0,
            pid: player.id().to_owned(),
            pname: player.name().to_owned(),
            active: true,
            position,
            remaining: player.bomb_time(),
            range: player.range(),
            timestamp: unix_timestamp(),
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn pid(&self) -> &str {
        self.pid.as_str()
    }

    pub fn pname(&self) -> &str {
        self.pname.as_str()
    }

    pub fn position(&self) -> MapPosition {
        self.position
    }

    pub fn tick(&mut self, delta_time: f32) -> Option<Explosion> {
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

impl TryFrom<serde_json::Value> for Bomb {
    type Error = JsonError;

    fn try_from(value: serde_json::Value) -> Result<Self, JsonError> {
        serde_json::from_value(value).map_err(|e| e.into())
    }
}

impl ToJson for Bomb {
    fn to_json(&self) -> Result<serde_json::Value, JsonError> {
        serde_json::to_value(self).map_err(|e| e.into())
    }
}

impl HasId for Bomb {
    fn set_id(&mut self, id: u32) {
        self.id = id;
    }
}
