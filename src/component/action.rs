use crate::traits::gameobject::{GameObject, SuperValue};
use serde_json::json;

#[derive(Default, Clone)]
pub struct Action {
    pub x: f32,
    pub y: f32,
    pub fire: bool,
    id: u32,
    pub deltatime: f32, // TODO: do I need this?
}

impl Action {
    pub fn new() -> Self {
        Action::default()
    }

    pub fn clear(&mut self) {
        self.x = 0.0;
        self.y = 0.0;
        self.fire = false;
        self.id = 0;
        self.deltatime = 0.0;
    }

    pub fn set(&mut self, x: f32, y: f32, fire: bool) {
        self.x = x;
        self.y = y;
        self.fire = fire;
    }
}

impl GameObject for Action {
    fn to_json(&self) -> serde_json::Value {
        json!({
            "id": self.id,
            "x": self.x,
            "y": self.y,
            "fire": self.fire,
            "deltaTime": self.deltatime
        })
    }

    fn from_json(&mut self, data: &serde_json::Value) {
        let sv = SuperValue::new(data);
        self.id = sv.get_u32("id");
        self.x = sv.get_f32("x");
        self.y = sv.get_f32("y");
        self.fire = sv.get_bool("fire");
        self.deltatime = sv.get_f32("deltaTime");
    }
}