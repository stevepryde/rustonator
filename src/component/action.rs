use crate::traits::jsonobject::{JSONObject, JSONValue};
use serde_json::json;

#[derive(Default, Clone)]
pub struct Action {
    x: i32,
    y: i32,
    fire: bool,
    id: u32,
    deltatime: f32, // TODO: do I need this?
}

impl Action {
    pub fn new() -> Self {
        Action::default()
    }

    pub fn is_empty(&self) -> bool {
        self.x == 0 && self.y == 0 && !self.fire
    }

    /// Force to -1, 0, 1
    fn clamp(val: i32) -> i32 {
        if val > 0 {
            1
        } else if val < 0 {
            -1
        } else {
            0
        }
    }

    pub fn get_x(&self) -> i32 {
        Self::clamp(self.x)
    }

    pub fn get_y(&self) -> i32 {
        Self::clamp(self.y)
    }

    pub fn clear(&mut self) {
        self.x = 0;
        self.y = 0;
        self.fire = false;
        self.id = 0;
        self.deltatime = 0.0;
    }

    pub fn set(&mut self, x: i32, y: i32, fire: bool) {
        self.x = Self::clamp(x);
        self.y = Self::clamp(y);
        self.fire = fire;
    }
}

impl JSONObject for Action {
    fn to_json(&self) -> serde_json::Value {
        json!({
            "id": self.id,
            "x": self.get_x(),
            "y": self.get_y(),
            "fire": self.fire,
            "deltaTime": self.deltatime
        })
    }

    fn from_json(&mut self, data: &serde_json::Value) {
        let sv = JSONValue::new(data);
        self.id = sv.get_u32("id");
        self.set(sv.get_i32("x"), sv.get_i32("y"), sv.get_bool("fire"));
        self.deltatime = sv.get_f32("deltaTime");
    }
}
