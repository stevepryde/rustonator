use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    x: i32,
    y: i32,
    fire: bool,
    id: u32,
    #[serde(rename = "deltaTime")]
    deltatime: f64, // TODO: do I need this?
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
