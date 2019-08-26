pub struct GameConfig {
    screen_x: u32,
    screen_y: u32,
}

impl Default for GameConfig {
    fn default() -> Self {
        GameConfig {
            screen_x: 800,
            screen_y: 600,
        }
    }
}

impl GameConfig {
    pub fn new() -> Self {
        GameConfig::default()
    }

    pub fn get_width(&self) -> u32 {
        self.screen_x
    }

    pub fn get_height(&self) -> u32 {
        self.screen_y
    }
}
