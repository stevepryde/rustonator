use crate::engine::explosion::Explosion;
use crate::engine::player::Player;
use crate::traits::jsonobject::{JSONObject, JSONValue};
use crate::utils::misc::unix_timestamp;
use serde_json::json;
use crate::engine::datatypes::MapPosition;
use crate::tools::itemstore::HasId;

// TODO: use type system for ids for better type safety.

pub struct Bomb {
    id: u32,
    pid: String,
    pname: String,
    active: bool,
    // Coordinates are cell coordinates, not pixels!
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

impl JSONObject for Bomb {
    fn to_json(&self) -> serde_json::Value {
        json!({
            "id": self.id,
            "pid": self.pid,
            "pname": self.pname,
            "active": self.active,
            "mapX": self.position.x,
            "mapY": self.position.y,
            "remaining": self.remaining,
            "range": self.range,
        })
    }

    fn from_json(&mut self, data: &serde_json::Value) {
        let sv = JSONValue::new(data);
        self.id = sv.get_u32("id");
        self.pid = sv.get_string("pid");
        self.pname = sv.get_string("pname");
        self.active = sv.get_bool("active");
        self.position = MapPosition::new(sv.get_u32("mapX"), sv.get_u32("mapY"));
        self.remaining = sv.get_f32("remaining");
        self.range = sv.get_u32("range");

        // NOTE: We lose timestamp in the client!
        // TODO: It would assist AI if access to the timestamp was granted. Do we want this?
        self.timestamp = 0;
    }
}

impl HasId for Bomb {
    fn set_id(&mut self, id: u32) {
        self.id = id;
    }
}
