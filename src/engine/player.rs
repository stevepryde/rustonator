use crate::{
    comms::playercomm::{PlayerComm, PlayerMessage},
    component::{
        action::Action,
        effect::{Effect, EffectType},
    },
    engine::{
        bomb::{BombRange, BombTime},
        position::PixelPositionF64,
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
use tokio::time::Instant;

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

    pub fn bomb_time(&self) -> BombTime {
        self.bomb_time
    }

    pub fn range(&self) -> BombRange {
        self.range
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

    pub fn ws(&self) -> &PlayerComm {
        &self.ws
    }

    pub fn update(&mut self, delta_time: f64) {
        let action = self.action.clone();
        self.update_with_temp_action(&action, delta_time);
    }

    pub fn terminate(&mut self) {
        self.state = PlayerState::Dead;
    }

    fn update_with_temp_action(&mut self, tmp_action: &Action, delta_time: f64) {
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
        self.position.x += tmp_action.get_x() as f64 * delta_time * effective_speed;
        self.position.y += tmp_action.get_y() as f64 * delta_time * effective_speed;
    }

    fn add_effect(&mut self, effect: Effect) {
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

    fn add_flag(&mut self, flag: PlayerFlags) {
        self.flags |= flag;
    }

    fn del_flag(&mut self, flag: PlayerFlags) {
        self.flags &= !flag;
    }

    fn has_flag(&self, flag: PlayerFlags) -> bool {
        self.flags.contains(flag)
    }

    fn set_position(&mut self, pos: PixelPositionF64) {
        self.position = pos;
    }

    fn set_action(&mut self, action: Action) {
        self.action = action;
    }

    fn add_random_effect(&mut self) -> String {
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
            Ok(Some(PlayerMessage::JoinGame(name))) => {
                info!("Player {:?} is joining with name '{}'", self.id(), name);
                self.set_name(&sanitise_name(&name));
                self.set_invincible();
                let spawn_point = world.get_spawn_point();
                self.set_position(PixelPositionF64::from_map_position(spawn_point, &world));

                let available_images = vec!["p1", "p2", "p3", "p4"];
                self.image = available_images
                    .choose(&mut rand::thread_rng())
                    .unwrap_or(&"p1")
                    .to_string();

                // Serialize here to avoid cloning both structures only to serialize later.
                self.ws
                    .send(PlayerMessage::SpawnPlayer(self.ser()?, world.data().ser()?))
                    .await?;

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
