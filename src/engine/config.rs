pub struct GameConfig {
    screen_x: u32,
    screen_y: u32
}

impl GameConfig {
    pub fn new() -> Self {
        GameConfig { screen_x: 800, screen_y: 600 }
    }
}