use crate::{
    component::action::Action,
    engine::{
        player::PlayerId,
        position::{MapPosition, PixelPositionF64, PositionOffset},
        types::PlayerList,
        world::World,
    },
    tools::itemstore::HasId,
    traits::{
        celltypes::{CanPass, CellType},
        randenum::RandEnumFrom,
    },
};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::ops::Add;

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
enum DirAction {
    Clockwise,
    Anticlockwise,
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

    fn get_offset(&self) -> PositionOffset {
        match self {
            MobTargetDir::Up => PositionOffset::new(0, -1),
            MobTargetDir::Right => PositionOffset::new(1, 0),
            MobTargetDir::Down => PositionOffset::new(0, 1),
            MobTargetDir::Left => PositionOffset::new(-1, 0),
        }
    }
}

impl Add<DirAction> for MobTargetDir {
    type Output = MobTargetDir;

    fn add(self, rhs: DirAction) -> Self::Output {
        match rhs {
            DirAction::Clockwise => self.right(),
            DirAction::Anticlockwise => self.left(),
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
    target_player: PlayerId,
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
                target_player: PlayerId::from(0),
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

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn terminate(&mut self) {
        self.active = false;
    }

    pub fn position(&self) -> PixelPositionF64 {
        self.position
    }

    pub fn set_position(&mut self, pos: PixelPositionF64) {
        self.position = pos;
    }

    pub fn is_smart(&self) -> bool {
        self.server_data.smart
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
        self.position.x += tmp_action.x() as f64 * delta_time * effective_speed;
        self.position.y += tmp_action.y() as f64 * delta_time * effective_speed;
    }

    pub fn choose_new_target(&mut self, world: &World, players: &PlayerList) {
        if self.server_data.danger {
            self.server_data.target_mode = MobTargetMode::DangerAvoidance;
        } else {
            self.server_data.target_mode = MobTargetMode::random();
        }
        let map_pos = self.position().to_map_position(world);

        let mut has_target = false;
        match self.server_data.target_mode {
            MobTargetMode::NearbyCell => {
                let blank = world.find_nearest_blank(map_pos.random_offset(self.server_data.range));
                if !blank.is_top_left() {
                    self.server_data.target_remaining = thread_rng().gen_range(5.0, 25.0);
                    self.server_data.target_position = blank;
                    has_target = true;
                }
            }
            MobTargetMode::NearbyPlayer => {
                for p in players.values() {
                    if p.position()
                        .to_map_position(world)
                        .is_within_range(map_pos, self.server_data.range as i32)
                    {
                        self.server_data.target_player = p.id();
                        self.server_data.target_remaining = thread_rng().gen_range(10.0, 120.0);
                        has_target = true;
                        break;
                    }
                }
            }
            MobTargetMode::Clockwise | MobTargetMode::Anticlockwise => {
                self.server_data.target_remaining = thread_rng().gen_range(1.0, 5.0);
                has_target = true;
            }
            MobTargetMode::ClockwiseNext | MobTargetMode::AnticlockwiseNext => {
                self.server_data.old_position = map_pos;
                self.server_data.target_remaining = thread_rng().gen_range(1.0, 10.0);
                has_target = true;
            }
            MobTargetMode::DangerAvoidance => {
                self.server_data.target_remaining = 99999.0;
                let safest =
                    world.path_find_nearest_safe_space(self, map_pos, self.server_data.range);
                self.server_data.target_position = safest;
                has_target = true;
            }
        }

        if !has_target {
            // Just assign a default - clockwise.
            self.server_data.old_position = map_pos;
            self.server_data.target_remaining = thread_rng().gen_range(1.0, 10.0);
        }
    }

    fn update_action(&mut self, delta_time: f64, players: &PlayerList, world: &World) {
        let map_pos = self.position().to_map_position(world);
        self.action.clear();

        let mut new_target = false;
        let mut dir_action: Option<DirAction> = None;
        let mut opportunistic = false;
        match self.server_data.target_mode {
            MobTargetMode::NearbyCell => {
                if map_pos == self.server_data.target_position {
                    // We've arrived. Choose a new one.
                    new_target = true;
                } else {
                    match world.path_find(
                        self,
                        map_pos,
                        self.server_data.target_position,
                        self.server_data.range * 2,
                    ) {
                        Some(best) => {
                            self.action.set(best.x, best.y, false);
                        }
                        None => {
                            new_target = true;
                        }
                    }
                }
            }
            MobTargetMode::NearbyPlayer => {
                if let Some(p) = players.get(&self.server_data.target_player) {
                    if p.is_dead() {
                        new_target = true;
                    } else {
                        match world.path_find(
                            self,
                            map_pos,
                            p.position().to_map_position(world),
                            self.server_data.range * 2,
                        ) {
                            Some(best) => {
                                self.action.set(best.x, best.y, false);
                            }
                            None => {
                                new_target = true;
                            }
                        }
                    }
                }
            }
            MobTargetMode::Clockwise => {
                dir_action = Some(DirAction::Clockwise);
            }
            MobTargetMode::Anticlockwise => {
                dir_action = Some(DirAction::Anticlockwise);
            }
            MobTargetMode::ClockwiseNext => {
                dir_action = Some(DirAction::Clockwise);
                opportunistic = true;
            }
            MobTargetMode::AnticlockwiseNext => {
                dir_action = Some(DirAction::Anticlockwise);
                opportunistic = true;
            }
            MobTargetMode::DangerAvoidance => {
                if world
                    .get_mob_data(self.server_data.target_position)
                    .is_some()
                {
                    // Still not safe, get new target.
                    let safest =
                        world.path_find_nearest_safe_space(self, map_pos, self.server_data.range);
                    self.server_data.target_position = safest;
                }

                // Go.
                if let Some(best) = world.path_find(
                    self,
                    map_pos,
                    self.server_data.target_position,
                    self.server_data.range * 2,
                ) {
                    self.action.set(best.x, best.y, false);
                }
            }
        }

        if let Some(da) = dir_action {
            let mut done = false;
            if opportunistic && map_pos != self.server_data.old_position {
                let new_dir = self.server_data.target_dir.clone() + da.clone();
                let offset = new_dir.get_offset();
                if self.can_pass_position(map_pos + offset, world) {
                    self.server_data.target_dir = new_dir;
                    self.server_data.old_position = map_pos;
                    self.action.set(offset.x, offset.y, false);
                    done = true;
                }
            }

            if !done {
                let offset = self.server_data.target_dir.get_offset();
                if self.can_pass_position(map_pos + offset, world) {
                    self.action.set(offset.x, offset.y, false);
                } else {
                    // There is a block here but we cannot pass.
                    self.server_data.target_dir = self.server_data.target_dir.clone() + da;
                }
            }
        }

        self.server_data.target_remaining -= delta_time;
        if self.server_data.target_remaining <= 0.0 || new_target {
            self.choose_new_target(world, players);
        }
    }

    fn danger_enable(&mut self, world: &World, players: &PlayerList) {
        self.server_data.danger = true;
        match self.server_data.target_mode {
            MobTargetMode::DangerAvoidance => {}
            _ => self.choose_new_target(world, players),
        }
    }

    fn danger_disable(&mut self, world: &World, players: &PlayerList) {
        self.server_data.danger = false;
        if let MobTargetMode::DangerAvoidance = self.server_data.target_mode {
            self.choose_new_target(world, players);
        }
    }

    pub fn update(&mut self, delta_time: f64, players: &PlayerList, world: &World) {
        if !self.is_active() {
            return;
        }

        let map_pos = self.position().to_map_position(world);
        if let Some(CellType::Wall) = world.get_cell(map_pos) {
            // Oops - we're in a wall. Reposition to nearby blank space.
            let blank = world.find_nearest_blank(map_pos);
            self.set_position(PixelPositionF64::from_map_position(blank, world));
        }

        // If we're in danger, do something about it.
        if self.server_data.danger {
            // We were in danger. Are we still in danger ?
            if world.get_mob_data(map_pos).is_none() {
                self.danger_disable(world, players);
            }
        } else {
            // We haven't been in danger but are we in danger now?
            if self.server_data.smart && world.get_mob_data(map_pos).is_some() {
                self.danger_enable(world, players);
            }
        }

        self.update_action(delta_time, players, world);
        let mut tmp_action = self.action.clone();
        // Try X movement.
        let try_pos = map_pos + PositionOffset::new(tmp_action.x(), 0);
        if !self.can_pass_position(try_pos, world) {
            // Can't pass horizontally, so lock X position.
            self.position.x = PixelPositionF64::from_map_position(try_pos, world).x;
        }
        // Try Y movement.
        let try_pos = map_pos + PositionOffset::new(0, tmp_action.y());
        if !self.can_pass_position(try_pos, world) {
            // Can't pass vertically, so lock Y position.
            self.position.y = PixelPositionF64::from_map_position(try_pos, world).y;
        }

        // Lock to gridlines.
        let tolerance = self.speed * delta_time;
        if tmp_action.x() != 0 {
            // Moving horizontally, make sure we're on a gridline.
            let target_y = PixelPositionF64::from_map_position(map_pos, world).y;
            if target_y > self.position.y + tolerance {
                tmp_action.setxy(0, 1);
            } else if target_y < self.position.y - tolerance {
                tmp_action.setxy(0, -1);
            } else {
                self.position.y = target_y;
                tmp_action.setxy(tmp_action.x(), 0);
            }
        } else if tmp_action.y() != 0 {
            // Moving vertically, make sure we're on a gridline.
            let target_x = PixelPositionF64::from_map_position(map_pos, world).x;
            if target_x > self.position.x + tolerance {
                tmp_action.setxy(1, 0);
            } else if target_x < self.position.x - tolerance {
                tmp_action.setxy(-1, 0);
            } else {
                self.position.x = target_x;
                tmp_action.setxy(0, tmp_action.y());
            }
        }

        self.update_with_temp_action(tmp_action, delta_time);
    }
}

impl HasId<MobId> for Mob {
    fn set_id(&mut self, id: MobId) {
        self.id = id;
    }
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
