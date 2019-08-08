use crate::engine::bomb::Bomb;
use crate::traits::gameobject::{GameObject, SuperValue};
use crate::utils::misc::unix_timestamp;
use serde_json::json;

pub struct Explosion {
    id: u32,
    pid: String,
    pname: String,
    active: bool,
    map_x: u32,
    map_y: u32,
    remaining: f32,
    harmful: bool,
    timestamp: i64,
}

impl Explosion {
    pub fn new(id: u32, bomb: Option<&Bomb>, map_x: u32, map_y: u32) -> Self {
        Explosion {
            id,
            pid: bomb.map_or(String::new(), |x| x.pid().to_owned()),
            pname: bomb.map_or(String::new(), |x| x.pname().to_owned()),
            active: true,
            map_x,
            map_y,
            remaining: 0.5,
            harmful: true,
            timestamp: unix_timestamp(),
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.remaining -= delta_time;
        if self.remaining <= 0.3 {
            self.harmful = false;
        }

        if self.remaining <= 0.0 {
            self.remaining = 0.0;
        }
    }
}

impl GameObject for Explosion {
    fn to_json(&self) -> serde_json::Value {
        json!({
            "id": self.id,
            "pid": self.pid,
            "pname": self.pname,
            "active": self.active,
            "mapX": self.map_x,
            "mapY": self.map_y,
            "remaining": self.remaining,
            "harmful": self.harmful
        })
    }

    fn from_json(&mut self, data: &serde_json::Value) {
        let sv = SuperValue::new(data);
        self.id = sv.get_u32("id");
        self.pid = sv.get_string("pid");
        self.pname = sv.get_string("pname");
        self.active = sv.get_bool("active");
        self.map_x = sv.get_u32("mapX");
        self.map_y = sv.get_u32("mapY");
        self.remaining = sv.get_f32("remaining");
        self.harmful = sv.get_bool("harmful");

        // NOTE: We lose timestamp in the client!
        // TODO: It would assist AI if access to the timestamp was granted. Do we want this?
        self.timestamp = 0;
    }
}
