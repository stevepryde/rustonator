use crate::{
    engine::{bomb::Bomb, position::MapPosition},
    tools::itemstore::HasId,
    traits::worldobject::{JsonError, ToJson},
    utils::misc::unix_timestamp,
};
use serde::{Deserialize, Serialize};
use serde_json;
use std::convert::TryFrom;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Explosion {
    id: u32,
    pid: String,
    pname: String,
    active: bool,
    #[serde(flatten)]
    position: MapPosition,
    remaining: f32,
    harmful: bool,
    timestamp: i64,
}

impl Explosion {
    pub fn new(bomb: Option<&Bomb>, position: MapPosition) -> Self {
        Explosion {
            id: 0,
            pid: bomb.map_or(String::new(), |x| x.pid().to_owned()),
            pname: bomb.map_or(String::new(), |x| x.pname().to_owned()),
            active: true,
            position,
            remaining: 0.5,
            harmful: true,
            timestamp: unix_timestamp(),
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn position(&self) -> MapPosition {
        self.position
    }

    pub fn update(&mut self, delta_time: f32) {
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

impl TryFrom<serde_json::Value> for Explosion {
    type Error = JsonError;

    fn try_from(value: serde_json::Value) -> Result<Self, JsonError> {
        serde_json::from_value(value).map_err(|e| e.into())
    }
}

impl ToJson for Explosion {
    fn to_json(&self) -> Result<serde_json::Value, JsonError> {
        serde_json::to_value(self).map_err(|e| e.into())
    }
}

impl HasId for Explosion {
    fn set_id(&mut self, id: u32) {
        self.id = id;
    }
}
