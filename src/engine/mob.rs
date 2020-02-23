use crate::{
    component::action::Action,
    engine::position::{MapPosition, PixelPositionF64},
    traits::{
        celltypes::{CanPass, CellType},
        randenum::RandEnumFrom,
    },
};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug)]
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

impl From<u8> for MobTargetMode {
    fn from(value: u8) -> Self {
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
}

// Provides MobTargetMode::random().
impl RandEnumFrom<u8> for MobTargetMode {
    fn get_enum_values() -> Vec<u8> {
        (0..7).collect()
    }
}

#[derive(Debug, Clone)]
pub enum MobTargetDir {
    Up = 0,
    Right = 1,
    Down = 2,
    Left = 3,
}

impl From<u8> for MobTargetDir {
    fn from(value: u8) -> Self {
        match value {
            0 => MobTargetDir::Up,
            1 => MobTargetDir::Right,
            2 => MobTargetDir::Down,
            3 => MobTargetDir::Left,
            _ => panic!("Invalid ModTargetDir: {}", value),
        }
    }
}

impl RandEnumFrom<u8> for MobTargetDir {
    fn get_enum_values() -> Vec<u8> {
        (0..4).collect()
    }
}

impl MobTargetDir {
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

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MobId(u64);

impl From<u64> for MobId {
    fn from(value: u64) -> Self {
        MobId(value)
    }
}

#[derive(Debug, Clone)]
pub struct MobServerData {
    target_mode: MobTargetMode,
    target_remaining: f64,

    // Position of current target. Used by NearbyCell mode.
    target_position: MapPosition,
    // Position when we switch direction, to prevent mob going in circles for modes 4 & 5.
    old_position: MapPosition,
    target_player: String, // pid.
    target_dir: MobTargetDir,
    range: u32,   // Visibility distance.
    smart: bool,  // Some bomb/explosion avoidance AI.
    danger: bool, // Triggers smart mob to GTFO.
}

#[derive(Debug, Clone, Serialize)]
pub struct Mob {
    id: MobId,
    active: bool,
    #[serde(flatten)]
    position: PixelPositionF64,
    action: Action,
    speed: f64,
    image: String,
    name: String,
    #[serde(skip)]
    server_data: MobServerData,
}

impl Default for Mob {
    fn default() -> Self {
        Mob {
            id: MobId::from(0),
            active: true,
            position: PixelPositionF64::new(0.0, 0.0),
            action: Action::new(),
            speed: 60.0, // pixels per second.
            image: String::from("mob1"),
            name: String::new(),

            // Server init.
            server_data: MobServerData {
                target_mode: MobTargetMode::NearbyCell,
                target_remaining: 0.0,
                target_position: MapPosition::new(0, 0),
                old_position: MapPosition::new(0, 0),
                target_player: String::new(),
                target_dir: MobTargetDir::Up,
                range: 8,
                smart: rand::thread_rng().gen_range(0, 10) > 7,
                danger: false,
            },
        }
    }
}

impl Mob {
    pub fn new() -> Self {
        Mob::default()
    }

    pub fn position(&self) -> PixelPositionF64 {
        self.position
    }

    pub fn update_with_temp_action(&mut self, tmp_action: Action, delta_time: f64) {
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
        self.position.x += tmp_action.get_x() as f64 * delta_time * effective_speed;
        self.position.y += tmp_action.get_y() as f64 * delta_time * effective_speed;
    }

    fn get_delta_for_dir(dir: MobTargetDir) -> (i32, i32) {
        match dir {
            MobTargetDir::Up => (0, -1),
            MobTargetDir::Right => (1, 0),
            MobTargetDir::Down => (0, 1),
            MobTargetDir::Left => (-1, 0),
        }
    }

    // fn choose_new_target(&mut self, world: &World, playerList: &PlayerList) {}
}

impl CanPass for Mob {
    fn can_pass(&self, cell_type: CellType) -> bool {
        match cell_type {
            CellType::Wall => false,
            CellType::Mystery => false,
            CellType::Bomb => false,
            _ => true,
        }
    }
}
