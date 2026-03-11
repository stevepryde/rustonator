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
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::ops::Add;

const MAX_MOVEMENT_STEP_SECONDS: f64 = 0.05;

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
        (0..6).collect()
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
                smart: rand::rng().random_range(0..10) > 7,
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

    pub fn update_with_temp_action(&mut self, tmp_action: &Action, delta_time: f64) {
        if tmp_action.is_empty() {
            return;
        }

        let effective_speed = self.speed.clamp(50.0, 300.0);
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
                    self.server_data.target_remaining = rand::rng().random_range(5.0..25.0);
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
                        self.server_data.target_remaining = rand::rng().random_range(5.0..120.0);
                        has_target = true;
                        break;
                    }
                }
            }
            MobTargetMode::Clockwise | MobTargetMode::Anticlockwise => {
                self.server_data.target_remaining = rand::rng().random_range(1.0..5.0);
                has_target = true;
            }
            MobTargetMode::ClockwiseNext | MobTargetMode::AnticlockwiseNext => {
                self.server_data.old_position = map_pos;
                self.server_data.target_remaining = rand::rng().random_range(1.0..10.0);
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
            self.server_data.target_mode = MobTargetMode::Clockwise;
            self.server_data.old_position = map_pos;
            self.server_data.target_remaining = rand::rng().random_range(1.0..10.0);
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
                } // Else we cannot find a path to a safe space! Probably
                  // trapped by player.
            }
        }

        if let Some(da) = dir_action {
            let mut done = false;
            if opportunistic && map_pos != self.server_data.old_position {
                let new_dir = self.server_data.target_dir.clone() + da.clone();
                let offset = new_dir.get_offset();
                if self.can_pass(map_pos + offset, world) {
                    self.server_data.target_dir = new_dir;
                    self.server_data.old_position = map_pos;
                    self.action.set(offset.x, offset.y, false);
                    done = true;
                }
            }

            if !done {
                let halftw = (world.sizes().tile_size().width as f64 / 2.0) - 1.0;
                let halfth = (world.sizes().tile_size().height as f64 / 2.0) - 1.0;
                let offset = self.server_data.target_dir.get_offset();
                let target_pos = PixelPositionF64::new(
                    self.position.x - (offset.x as f64 * halftw),
                    self.position.y - (offset.y as f64 * halfth),
                );
                let target_map_pos = target_pos.to_map_position(world) + offset;
                if self.can_pass(target_map_pos, world) {
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

        let step_count = (delta_time / MAX_MOVEMENT_STEP_SECONDS).ceil().max(1.0) as usize;
        let step_delta = delta_time / step_count as f64;
        for _ in 0..step_count {
            self.ensure_valid_position(world);
            self.update_movement_step(step_delta, players, world);
            self.ensure_valid_position(world);
        }
    }

    fn update_movement_step(&mut self, delta_time: f64, players: &PlayerList, world: &World) {
        let map_pos = self.position().to_map_position(world);
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
        if tmp_action.x() != 0 {
            let try_pos = map_pos + PositionOffset::new(tmp_action.x(), 0);
            if !self.can_pass(try_pos, world) {
                // Can't pass horizontally, so lock X position.
                let target_x = PixelPositionF64::from_map_position(map_pos, world).x;
                if (tmp_action.x() < 0 && self.position.x <= target_x)
                    || (tmp_action.x() > 0 && self.position.x >= target_x)
                {
                    self.position.x = target_x;
                    tmp_action.setxy(0, tmp_action.y());
                }
            }
        }

        if tmp_action.y() != 0 {
            // Try Y movement.
            let try_pos = map_pos + PositionOffset::new(0, tmp_action.y());
            if !self.can_pass(try_pos, world) {
                // Can't pass vertically, so lock Y position.
                let target_y = PixelPositionF64::from_map_position(map_pos, world).y;
                if (tmp_action.y() < 0 && self.position.y <= target_y)
                    || (tmp_action.y() > 0 && self.position.y >= target_y)
                {
                    self.position.y = target_y;
                    tmp_action.setxy(tmp_action.x(), 0);
                }
            }
        }

        // Lock to gridlines.
        let tolerance = world.sizes().tile_size().width as f64 * 0.3;
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

        self.update_with_temp_action(&tmp_action, delta_time);
        self.fix_position_and_tmpaction(&mut tmp_action, map_pos, world);
    }

    fn ensure_valid_position(&mut self, world: &World) {
        let map_pos = self.position().to_map_position(world);
        if self.is_collision_blocked_cell(world.get_cell(map_pos)) {
            let blank = world.find_nearest_blank(map_pos);
            self.set_position(PixelPositionF64::from_map_position(blank, world));
        }
    }

    fn is_collision_blocked_cell(&self, cell: Option<CellType>) -> bool {
        matches!(
            cell,
            Some(CellType::Wall) | Some(CellType::Mystery) | Some(CellType::Bomb) | None
        )
    }

    fn fix_position_and_tmpaction(
        &mut self,
        tmp_action: &mut Action,
        map_pos: MapPosition,
        world: &World,
    ) {
        if tmp_action.x() != 0 {
            let try_pos = map_pos + PositionOffset::new(tmp_action.x(), 0);
            if !self.can_pass(try_pos, world) {
                let target_x = PixelPositionF64::from_map_position(map_pos, world).x;
                if (tmp_action.x() < 0 && self.position.x <= target_x)
                    || (tmp_action.x() > 0 && self.position.x >= target_x)
                {
                    self.position.x = target_x;
                    tmp_action.setxy(0, tmp_action.y());
                }
            }
        }

        if tmp_action.y() != 0 {
            let try_pos = map_pos + PositionOffset::new(0, tmp_action.y());
            if !self.can_pass(try_pos, world) {
                let target_y = PixelPositionF64::from_map_position(map_pos, world).y;
                if (tmp_action.y() < 0 && self.position.y <= target_y)
                    || (tmp_action.y() > 0 && self.position.y >= target_y)
                {
                    self.position.y = target_y;
                    tmp_action.setxy(tmp_action.x(), 0);
                }
            }
        }
    }
}

impl HasId<MobId> for Mob {
    fn set_id(&mut self, id: MobId) {
        self.id = id;
    }
}

impl CanPass for Mob {
    fn can_pass(&self, position: MapPosition, world: &World) -> bool {
        match world.get_cell(position) {
            Some(CellType::Wall) | Some(CellType::Mystery) | Some(CellType::Bomb) => false,
            _ => {
                if self.is_smart() && !self.server_data.danger {
                    // Check for danger!
                    world.get_mob_data(position).is_none()
                } else {
                    true
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        engine::{bomb::Bomb, config::GameConfig, player::{Player, PlayerId}, types::BombList, world::World},
        comms::playercomm::PlayerComm,
    };
    use tokio::sync::mpsc;

    fn test_player() -> Player {
        let (tx, _rx_external) = mpsc::channel(1);
        let (_tx_external, rx) = mpsc::channel(1);
        let mut player = Player::new(PlayerId::from(1), PlayerComm::new(PlayerId::from(1), tx, rx));
        player.set_name("test");
        player
    }

    #[test]
    fn mob_does_not_tunnel_through_bombs_on_large_delta() {
        let config = GameConfig::new();
        let mut world = World::new(15, 15, &config);
        let mut bombs = BombList::new();
        let player = test_player();
        let players = PlayerList::new();
        let mut mob = Mob::new();

        mob.set_position(PixelPositionF64::new(112.0, 48.0));
        mob.action.set(1, 0, false);
        mob.server_data.target_remaining = 999.0;
        mob.server_data.target_mode = MobTargetMode::Clockwise;
        mob.server_data.target_dir = MobTargetDir::Right;
        mob.server_data.old_position = MapPosition::new(3, 1);

        world.add_bomb(Bomb::new(&player, MapPosition::new(5, 1), false), &mut bombs);

        mob.update(0.8, &players, &world);

        let final_pos = mob.position().to_map_position(&world);
        assert_ne!(final_pos, MapPosition::new(5, 1));
        assert!(final_pos.x <= 4 || final_pos.y != 1);
    }

    #[test]
    fn mob_repositions_out_of_bomb_tiles() {
        let config = GameConfig::new();
        let mut world = World::new(15, 15, &config);
        let mut bombs = BombList::new();
        let player = test_player();
        let players = PlayerList::new();
        let mut mob = Mob::new();

        mob.set_position(PixelPositionF64::new(112.0, 48.0));
        world.add_bomb(Bomb::new(&player, MapPosition::new(3, 1), false), &mut bombs);

        mob.update(0.0, &players, &world);

        assert_ne!(mob.position().to_map_position(&world), MapPosition::new(3, 1));
    }
}
