use crate::component::action::Action;
use crate::component::effect::{Effect, EffectType};
use crate::traits::celltypes::{CanPass, CellType};
use crate::traits::gameobject::{GameObject, SuperValue};
use rand::Rng;
use serde_json::json;

pub enum MobTargetMode {
    // Pick a nearby spot and try to reach it.
    NearbyCell = 0,
    // Pick a nearby player and try to follow them.
    NearbyPlayer = 1,
    // Always try moves in clockwise direction, starting with current dir.
    Clockwise = 2,
    // Always try moves in counter-clockwise direction, starting with current dir.
    Anticlockwise = 3,
    // Same as 2, but start at direction after current.
    ClockwiseNext = 4,
    // Same as 3, but start at direction after current.
    AnticlockwiseNext = 5,
    // Avoid danger (bomb nearby!)
    DangerAvoidance = 6,
}

impl MobTargetMode {
    pub fn from(value: u8) -> Self {
        match value {
            0 => MobTargetMode::NearbyCell,
            1 => MobTargetMode::NearbyPlayer,
            2 => MobTargetMode::Clockwise,
            3 => MobTargetMode::Anticlockwise,
            4 => MobTargetMode::ClockwiseNext,
            5 => MobTargetMode::AnticlockwiseNext,
            6 => MobTargetMode::DangerAvoidance,
            _ => panic!("Invalid mob target mode: {}", value),
        }
    }

    pub fn random() -> Self {
        // Don't include danger mode when getting random mode.
        MobTargetMode::from(rand::thread_rng().gen_range(0, 6))
    }
}

pub enum MobTargetDir {
    Up = 0,
    Right = 1,
    Down = 2,
    Left = 3,
}

impl MobTargetDir {
    pub fn from(value: u8) -> Self {
        match value {
            0 => MobTargetDir::Up,
            1 => MobTargetDir::Right,
            2 => MobTargetDir::Down,
            3 => MobTargetDir::Left,
            _ => panic!("Invalid ModTargetDir: {}", value),
        }
    }

    pub fn random() -> Self {
        // Don't include danger mode when getting random mode.
        MobTargetDir::from(rand::thread_rng().gen_range(0, 4))
    }

    pub fn right(self) -> Self {
        match self {
            MobTargetDir::Up => MobTargetDir::Right,
            MobTargetDir::Right => MobTargetDir::Down,
            MobTargetDir::Down => MobTargetDir::Left,
            MobTargetDir::Left => MobTargetDir::Up,
        }
    }

    pub fn left(self) -> Self {
        match self {
            MobTargetDir::Up => MobTargetDir::Left,
            MobTargetDir::Right => MobTargetDir::Up,
            MobTargetDir::Down => MobTargetDir::Right,
            MobTargetDir::Left => MobTargetDir::Down,
        }
    }
}

pub struct Mob {
    id: u32,
    active: bool,
    x: f32,
    y: f32,
    action: Action,
    speed: f32,
    image: String,
    name: String,

    // Server only.
    target_mode: MobTargetMode,
    target_remaining: f32,

    // Position of current target. Used by NearbyCell mode.
    target_mapx: u32,
    target_mapy: u32,
    // Position when we switch direction, to prevent mob going in circles for modes 4 & 5.
    old_mapx: u32,
    old_mapy: u32,
    target_player: String, // pid.
    target_dir: MobTargetDir,
    range: u32,   // Visibility distance.
    smart: bool,  // Some bomb/explosion avoidance AI.
    danger: bool, // Triggers smart mob to GTFO.
}

impl Default for Mob {
    fn default() -> Self {
        Mob {
            id: 0,
            active: true,
            x: 0.0,
            y: 0.0,
            action: Action::new(),
            speed: 60.0, // pixels per second.
            image: String::from("mob1"),
            name: String::new(),

            // Server init.
            target_mode: MobTargetMode::NearbyCell,
            target_remaining: 0.0,
            target_mapx: 0,
            target_mapy: 0,
            old_mapx: 0,
            old_mapy: 0,
            target_player: String::new(),
            target_dir: MobTargetDir::Up,
            range: 8,
            smart: rand::thread_rng().gen_range(0, 10) > 7,
            danger: false,
        }
    }
}

impl Mob {
    pub fn new() -> Self {
        Mob::default()
    }

    pub fn update_with_temp_action(&mut self, tmp_action: Action, delta_time: f32) {
        if tmp_action.is_empty() {
            return;
        }

        let effective_speed = if self.speed < 50.0 {
            50.0
        } else if self.speed > 300.0 {
            300.0
        } else {
            self.speed
        };
        self.x += tmp_action.get_x() as f32 * delta_time * effective_speed;
        self.y += tmp_action.get_y() as f32 * delta_time * effective_speed;
    }

    fn get_delta_for_dir(dir: MobTargetDir) -> (i32, i32) {
        match dir {
            MobTargetDir::Up => (0, -1),
            MobTargetDir::Right => (1, 0),
            MobTargetDir::Down => (0, 1),
            MobTargetDir::Left => (-1, 0),
        }
    }

    // fn choose_new_target(&mut self, world: &World, playerList: &PlayerList) {
    //     // TODO: ...
    // }
}

impl CanPass for Mob {
    fn can_pass(&self, cell_type: &CellType) -> bool {
        match cell_type {
            CellType::Wall => false,
            CellType::Mystery => false,
            CellType::Bomb => false,
            _ => true,
        }
    }
}

impl GameObject for Mob {
    fn to_json(&self) -> serde_json::Value {
        json!({
            "id": self.id,
            "active": self.active,
            "x": self.x,
            "y": self.y,
            "action": self.action.to_json(),
            "speed": self.speed,
            "image": self.image,
            "name": self.name
        })
    }

    fn from_json(&mut self, data: &serde_json::Value) {
        let sv = SuperValue::new(data);
        self.id = sv.get_u32("id");
        self.active = sv.get_bool("active");
        self.x = sv.get_f32("x");
        self.y = sv.get_f32("y");
        self.action.from_json(sv.get_value("action"));
        self.speed = sv.get_f32("speed");
        self.image = sv.get_string("image");
        self.name = sv.get_string("name");
    }
}
