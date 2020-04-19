use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    x: i32,
    y: i32,
    fire: bool,
    id: u32,
    #[serde(rename = "deltaTime")]
    delta_time: f64,
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
        match val {
            x if x > 0 => 1,
            x if x < 0 => -1,
            _ => 0,
        }
    }

    pub fn x(&self) -> i32 {
        Self::clamp(self.x)
    }

    pub fn y(&self) -> i32 {
        Self::clamp(self.y)
    }

    pub fn fire(&self) -> bool {
        self.fire
    }

    pub fn cease_fire(&mut self) {
        self.fire = false;
    }

    pub fn clear(&mut self) {
        self.x = 0;
        self.y = 0;
        self.fire = false;
        // self.id = 0;
        self.delta_time = 0.0;
    }

    pub fn set(&mut self, x: i32, y: i32, fire: bool) {
        self.x = Self::clamp(x);
        self.y = Self::clamp(y);
        self.fire = fire;
    }

    pub fn setxy(&mut self, x: i32, y: i32) {
        self.x = Self::clamp(x);
        self.y = Self::clamp(y);
    }

    pub fn set_dt(&mut self, delta_time: f64) {
        self.delta_time = delta_time;
    }
}
