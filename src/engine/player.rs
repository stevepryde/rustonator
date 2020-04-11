use crate::{
    comms::playercomm::{PlayerComm, PlayerMessage},
    component::{
        action::Action,
        effect::{Effect, EffectType},
    },
    engine::{
        bomb::{BombRange, BombTime},
        position::{PixelPositionF64, PositionOffset},
        world::World,
    },
    error::{ZError, ZResult},
    traits::{
        celltypes::{CanPass, CellType},
        randenum::RandEnumFrom,
    },
    utils::misc::Timestamp,
};
use bitflags::bitflags;
use log::*;
use rand::{seq::SliceRandom, Rng};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::convert::TryFrom;

bitflags! {
    #[derive(Default, Serialize, Deserialize)]
    pub struct PlayerFlags: u32 {
        const WALK_THROUGH_BOMBS = 0b0001;
        const INVINCIBLE         = 0b0010;
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PlayerId(u64);

impl From<u64> for PlayerId {
    fn from(value: u64) -> Self {
        PlayerId(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SerPlayer(Value);

impl TryFrom<&Player> for SerPlayer {
    type Error = ZError;

    fn try_from(player: &Player) -> ZResult<Self> {
        Ok(SerPlayer(serde_json::to_value(player)?))
    }
}

#[derive(Debug)]
pub enum PlayerState {
    Active,
    Joining,
    Dead,
}

#[derive(Debug, Serialize)]
pub struct Player {
    id: PlayerId,
    #[serde(skip)]
    state: PlayerState,
    #[serde(flatten)]
    position: PixelPositionF64,
    action: Action,
    speed: f64,
    image: String,
    range: BombRange,
    bomb_time: BombTime,
    max_bombs: u32,
    cur_bombs: u32,
    flags: PlayerFlags,
    score: u32,
    name: String,
    rank: u32,
    effects: Vec<Effect>,
    #[serde(skip)]
    effects_cache: Vec<Effect>,
    #[serde(skip)]
    last_time: Timestamp,
    #[serde(skip)]
    ws: PlayerComm,
}

impl Player {
    pub fn new(id: PlayerId, comm: PlayerComm) -> Self {
        Player {
            id,
            state: PlayerState::Joining,
            position: PixelPositionF64::new(0.0, 0.0),
            action: Action::new(),
            speed: 200.0,
            image: String::from("p1"),
            range: BombRange::from(1),
            bomb_time: BombTime::from(3.0),
            max_bombs: 1,
            cur_bombs: 0,
            flags: PlayerFlags::default(),
            score: 0,
            name: String::new(),
            rank: 0,
            effects: Vec::new(),
            effects_cache: Vec::new(),
            last_time: Timestamp::new(),
            ws: comm,
        }
    }

    pub fn ser(&self) -> ZResult<SerPlayer> {
        SerPlayer::try_from(self)
    }

    pub fn id(&self) -> PlayerId {
        self.id
    }

    pub fn is_dead(&self) -> bool {
        if let PlayerState::Dead = self.state {
            true
        } else {
            false
        }
    }

    pub fn has_joined(&self) -> bool {
        if let PlayerState::Joining = self.state {
            false
        } else {
            true
        }
    }

    pub fn state(&self) -> &PlayerState {
        &self.state
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    pub fn position(&self) -> PixelPositionF64 {
        self.position
    }

    pub fn position_mut(&mut self) -> &mut PixelPositionF64 {
        &mut self.position
    }

    pub fn set_position(&mut self, pos: PixelPositionF64) {
        self.position = pos;
    }

    pub fn action(&self) -> &Action {
        &self.action
    }

    pub fn action_mut(&mut self) -> &mut Action {
        &mut self.action
    }

    fn set_action(&mut self, action: Action) {
        self.action = action;
    }

    pub fn speed(&self) -> f64 {
        self.speed
    }

    pub fn score(&self) -> u32 {
        self.score
    }

    pub fn increase_score(&mut self, amount: u32) {
        self.score += amount;
    }

    pub fn decrease_score(&mut self, amount: u32) {
        if self.score > amount {
            self.score -= amount;
        } else {
            self.score = 0;
        }
    }

    pub fn bomb_time(&self) -> BombTime {
        self.bomb_time
    }

    pub fn increase_bomb_time(&mut self) {
        self.bomb_time += 1.0;
    }

    pub fn decrease_bomb_time(&mut self) {
        if self.bomb_time > BombTime::from(2.0) {
            self.bomb_time -= 1.0;
        }
    }

    pub fn range(&self) -> BombRange {
        self.range
    }

    pub fn increase_range(&mut self) {
        self.range += 1;
    }

    pub fn decrease_range(&mut self) {
        if self.range > BombRange::from(1) {
            self.range -= 1;
        }
    }

    pub fn max_bombs(&self) -> u32 {
        self.max_bombs
    }

    pub fn cur_bombs(&self) -> u32 {
        self.cur_bombs
    }

    pub fn has_bomb_remaining(&self) -> bool {
        self.cur_bombs < self.max_bombs
    }

    pub fn bomb_placed(&mut self) {
        self.cur_bombs += 1;
    }

    pub fn bomb_exploded(&mut self) {
        if self.cur_bombs > 0 {
            self.cur_bombs -= 1;
        }
    }

    pub fn increase_max_bombs(&mut self) {
        self.max_bombs += 1;
    }

    pub fn decrease_max_bombs(&mut self) {
        if self.max_bombs > 0 {
            self.max_bombs -= 1;
        }
    }

    pub fn ws(&mut self) -> &mut PlayerComm {
        &mut self.ws
    }

    pub fn terminate(&mut self) {
        self.state = PlayerState::Dead;
    }

    pub fn update_with_temp_action(&mut self, tmp_action: &Action, delta_time: f64) {
        std::mem::swap(&mut self.effects, &mut self.effects_cache);
        self.effects.clear();
        while !self.effects_cache.is_empty() {
            if let Some(x) = self.effects_cache.pop() {
                if x.active {
                    self.effects.push(x);
                } else {
                    self.undo_effect(&x);
                }
            }
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

    pub fn add_effect(&mut self, effect: Effect) {
        let mut effect = effect;
        match effect.effect_type {
            EffectType::SpeedUp => {
                self.speed += 50.0;
                effect.name = String::from(">>");
            }
            EffectType::SlowDown => {
                self.speed -= 50.0;
                effect.name = String::from("<<");
            }
            EffectType::Invincibility => {
                self.add_flag(PlayerFlags::INVINCIBLE);
            }
        }
        self.effects.push(effect);
    }

    fn undo_effect(&mut self, effect: &Effect) {
        match effect.effect_type {
            EffectType::SpeedUp => {
                self.speed -= 50.0;
            }
            EffectType::SlowDown => {
                self.speed += 50.0;
            }
            EffectType::Invincibility => {
                self.del_flag(PlayerFlags::INVINCIBLE);
            }
        }
    }

    pub fn add_flag(&mut self, flag: PlayerFlags) {
        self.flags |= flag;
    }

    pub fn del_flag(&mut self, flag: PlayerFlags) {
        self.flags &= !flag;
    }

    pub fn has_flag(&self, flag: PlayerFlags) -> bool {
        self.flags.contains(flag)
    }

    pub fn add_random_effect(&mut self) -> String {
        let effect = Effect::new(
            EffectType::random(),
            rand::thread_rng().gen_range(3.0f64, 8.0f64),
        );
        let name = effect.name.clone();
        self.add_effect(effect);
        name
    }

    pub fn set_invincible(&mut self) {
        let effect = Effect::new(EffectType::Invincibility, 5.0);
        self.add_effect(effect);
    }

    pub async fn handle_player_input(&mut self, world: &mut World) -> ZResult<bool> {
        if !self.has_joined() {
            return self.handle_player_join(world).await;
        }

        self.action.clear();
        match self.ws.recv_one().await {
            Ok(None) => {
                // No message waiting.
                Ok(true)
            }
            Ok(Some(PlayerMessage::Action(a))) => {
                info!("Player {:?} action received {:?}", self.id(), a);
                self.set_action(a);
                Ok(true)
            }
            Ok(x) => {
                error!("Player {:?} invalid message received: {:?}", self.id(), x);
                self.terminate();
                Ok(false)
            }
            Err(e) => {
                error!("Player {:?} error {:?}", self.id(), e);
                self.terminate();
                Ok(false)
            }
        }
    }

    pub async fn handle_player_join(&mut self, world: &mut World) -> ZResult<bool> {
        match self.ws.recv_one().await {
            Ok(None) => {
                // No message waiting.
                Ok(true)
            }
            Ok(Some(PlayerMessage::JoinGame(name))) => {
                info!("Player {:?} is joining with name '{}'", self.id(), name);
                self.set_name(&sanitise_name(&name));
                self.set_invincible();
                let spawn_point = world.get_spawn_point();
                self.set_position(PixelPositionF64::from_map_position(spawn_point, &world));

                let available_images = vec!["p1", "p2", "p3", "p4"];
                self.image = (*available_images
                    .choose(&mut rand::thread_rng())
                    .unwrap_or(&"p1"))
                .to_string();

                self.state = PlayerState::Active;
                // Serialize here to avoid cloning both structures only to serialize later.
                self.ws
                    .send(PlayerMessage::SpawnPlayer(self.ser()?, world.data().ser()?))
                    .await?;

                Ok(true)
            }
            Ok(x) => {
                error!(
                    "Player {:?} invalid join message received: {:?}",
                    self.id(),
                    x
                );
                self.terminate();
                Ok(false)
            }
            Err(e) => {
                error!("Player {:?} error {:?}", self.id(), e);
                self.terminate();
                Ok(false)
            }
        }
    }

    pub async fn got_item(&mut self, item: CellType) -> ZResult<bool> {
        match item {
            CellType::ItemBomb => {
                self.increase_max_bombs();
                self.ws().send_powerup("+B").await?;
                Ok(true)
            }
            CellType::ItemRange => {
                self.increase_range();
                self.ws().send_powerup("+R").await?;
                Ok(true)
            }
            CellType::ItemRandom => {
                let r: u8 = rand::thread_rng().gen_range(0, 10);
                let mut powerup_name = String::new();
                match r {
                    0 => {
                        if self.max_bombs() < 6 {
                            self.increase_max_bombs();
                            powerup_name = "+B".to_owned();
                        }
                    }
                    1 => {
                        if self.max_bombs() > 1 {
                            self.decrease_max_bombs();
                            powerup_name = "-B".to_owned();
                        }
                    }
                    2 => {
                        if self.range() < BombRange::from(8) {
                            self.increase_range();
                            powerup_name = "+R".to_owned();
                        }
                    }
                    3 => {
                        if self.range() > BombRange::from(1) {
                            self.decrease_range();
                            powerup_name = "-R".to_owned();
                        }
                    }
                    4 => {
                        if self.has_flag(PlayerFlags::WALK_THROUGH_BOMBS) {
                            self.del_flag(PlayerFlags::WALK_THROUGH_BOMBS);
                            powerup_name = "-TB".to_owned();
                        } else {
                            self.add_flag(PlayerFlags::WALK_THROUGH_BOMBS);
                            powerup_name = "+TB".to_owned();
                        }
                    }
                    5 => {
                        if self.bomb_time() < BombTime::from(4.0) {
                            self.increase_bomb_time();
                            powerup_name = "SB".to_owned();
                        }
                    }
                    6 => {
                        if self.bomb_time() > BombTime::from(2.0) {
                            self.decrease_bomb_time();
                            powerup_name = "FB".to_owned();
                        }
                    }
                    7 => {
                        if self.score() > 100 {
                            let pwrup: u32 = rand::thread_rng().gen_range(1, 10) * 10;
                            self.decrease_score(pwrup);
                            powerup_name = "-$".to_owned();
                        }
                    }
                    8 => {
                        let pwrup: u32 = rand::thread_rng().gen_range(1, 10) * 10;
                        self.increase_score(pwrup);
                        powerup_name = "+$".to_owned();
                    }
                    _ => powerup_name = self.add_random_effect(),
                }

                if powerup_name.is_empty() {
                    powerup_name = self.add_random_effect();
                }

                self.ws().send_powerup(&powerup_name).await?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn update(&mut self, world: &World, delta_time: f64) {
        let map_pos = self.position().to_map_position(&world);
        if let Some(CellType::Wall) = world.get_cell(map_pos) {
            // Oops - we're in a wall. Reposition to nearby blank space.
            let blank = world.find_nearest_blank(map_pos);
            self.set_position(PixelPositionF64::from_map_position(blank, &world));
        }

        let mut tmp_action = self.action().clone();
        // Try X movement.
        let try_pos = map_pos + PositionOffset::new(tmp_action.x(), 0);
        if !self.can_pass_position(try_pos, &world) {
            // Can't pass horizontally, so lock X position.
            self.position_mut().x = PixelPositionF64::from_map_position(try_pos, &world).x;
        }
        // Try Y movement.
        let try_pos = map_pos + PositionOffset::new(0, tmp_action.y());
        if !self.can_pass_position(try_pos, &world) {
            // Can't pass vertically, so lock Y position.
            self.position_mut().y = PixelPositionF64::from_map_position(try_pos, &world).y;
        }

        // Lock to gridlines.
        let tolerance = self.speed() * delta_time;
        if tmp_action.x() != 0 {
            // Moving horizontally, make sure we're on a gridline.
            let target_y = PixelPositionF64::from_map_position(map_pos, &world).y;
            if target_y > self.position().y + tolerance {
                tmp_action.setxy(0, 1);
            } else if target_y < self.position().y - tolerance {
                tmp_action.setxy(0, -1);
            } else {
                self.position().y = target_y;
                tmp_action.setxy(tmp_action.x(), 0);
            }
        } else if tmp_action.y() != 0 {
            // Moving vertically, make sure we're on a gridline.
            let target_x = PixelPositionF64::from_map_position(map_pos, &world).x;
            if target_x > self.position().x + tolerance {
                tmp_action.setxy(1, 0);
            } else if target_x < self.position().x - tolerance {
                tmp_action.setxy(-1, 0);
            } else {
                self.position().x = target_x;
                tmp_action.setxy(0, tmp_action.y());
            }
        }

        self.update_with_temp_action(&tmp_action, delta_time);
    }
}

impl CanPass for Player {
    fn can_pass(&self, cell_type: CellType) -> bool {
        match cell_type {
            CellType::Wall => false,
            CellType::Mystery => false,
            CellType::Bomb => self.has_flag(PlayerFlags::WALK_THROUGH_BOMBS),
            _ => true,
        }
    }
}

fn sanitise_name(name: &str) -> String {
    let r = Regex::new(r"^[^\w\s,._:'!^*()=\-]+$").unwrap();
    r.replace_all(name, "")
        .to_string()
        .chars()
        .take(30)
        .collect()
}
