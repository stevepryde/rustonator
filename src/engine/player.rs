use crate::{
    component::{
        action::Action,
        effect::{Effect, EffectType},
    },
    engine::position::PixelPositionF64,
    traits::{
        celltypes::{CanPass, CellType},
        randenum::RandEnumFrom,
        worldobject::{JsonError, ToJson},
    },
};
use bitflags::bitflags;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json;
use std::convert::TryFrom;

bitflags! {
    #[derive(Default, Serialize, Deserialize)]
    pub struct PlayerFlags: u32 {
        const WALK_THROUGH_BOMBS = 0b0001;
        const INVINCIBLE         = 0b0010;
    }
}

#[derive(Debug, Clone)]
pub struct Player {
    id: String,
    active: bool,
    position: PixelPositionF64,
    action: Action,
    speed: f64,
    image: String,
    range: u32,
    bomb_time: f64,
    max_bombs: u32,
    cur_bombs: u32,
    flags: PlayerFlags,
    score: u32,
    name: String,
    rank: u32,
    effects: [Vec<Effect>; 2],
    effect_index: usize,
    last_time: f64,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            id: String::new(),
            active: true,
            position: PixelPositionF64::new(0.0, 0.0),
            action: Action::new(),
            speed: 200.0,
            image: String::from("p1"),
            range: 1,
            bomb_time: 3.0,
            max_bombs: 1,
            cur_bombs: 0,
            flags: PlayerFlags::default(),
            score: 0,
            name: String::new(),
            rank: 0,
            effects: [Vec::new(), Vec::new()],
            effect_index: 0,
            last_time: 0.0,
        }
    }
}

impl Player {
    pub fn new() -> Self {
        Player::default()
    }

    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn position(&self) -> PixelPositionF64 {
        self.position
    }

    pub fn bomb_time(&self) -> f64 {
        self.bomb_time
    }

    pub fn range(&self) -> u32 {
        self.range
    }

    pub fn update(&mut self, delta_time: f64) {
        let action = self.action.clone();
        self.update_with_temp_action(&action, delta_time);
    }

    fn update_with_temp_action(&mut self, tmp_action: &Action, delta_time: f64) {
        let src_index = self.effect_index;
        let target_index = !self.effect_index;

        self.effects[target_index].clear();
        while !self.effects[src_index].is_empty() {
            if let Some(x) = self.effects[src_index].pop() {
                if x.active {
                    self.effects[target_index].push(x);
                } else {
                    self.undo_effect(&x);
                }
            }
        }

        self.effect_index = target_index;

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
        self.effects[self.effect_index].push(effect);
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

    fn set_invincible(&mut self) {
        let effect = Effect::new(EffectType::Invincibility, 5.0);
        self.add_effect(effect);
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerData {
    id: String,
    active: bool,
    x: f64,
    y: f64,
    action: Action,
    speed: f64,
    image: String,
    range: u32,
    bomb_time: f64,
    max_bombs: u32,
    cur_bombs: u32,
    flags: PlayerFlags,
    score: u32,
    name: String,
    rank: u32,
    effects: Vec<serde_json::Value>,
    last_time: f64,
}

impl TryFrom<serde_json::Value> for Player {
    type Error = JsonError;

    fn try_from(value: serde_json::Value) -> Result<Self, JsonError> {
        let data: PlayerData = serde_json::from_value(value)?;
        let mut effects = Vec::new();
        for v in data.effects.into_iter() {
            effects.push(Effect::try_from(v)?);
        }

        Ok(Player {
            id: data.id,
            active: data.active,
            position: PixelPositionF64::new(data.x, data.y),
            action: data.action,
            speed: data.speed,
            image: data.image,
            range: data.range,
            bomb_time: data.bomb_time,
            max_bombs: data.max_bombs,
            cur_bombs: data.cur_bombs,
            flags: data.flags,
            score: data.score,
            name: data.name,
            rank: data.rank,
            effects: [effects, Vec::new()],
            effect_index: 0,
            last_time: data.last_time,
            ..Default::default()
        })
    }
}

impl ToJson for Player {
    fn to_json(&self) -> Result<serde_json::Value, JsonError> {
        let mut effect_data = Vec::new();
        for effect in &self.effects[self.effect_index] {
            effect_data.push(effect.to_json()?);
        }

        let data = PlayerData {
            id: self.id.clone(),
            active: self.active,
            x: self.position.x,
            y: self.position.y,
            action: self.action.clone(),
            speed: self.speed,
            image: self.image.clone(),
            range: self.range,
            bomb_time: self.bomb_time,
            max_bombs: self.max_bombs,
            cur_bombs: self.cur_bombs,
            flags: self.flags,
            score: self.score,
            name: self.name.clone(),
            rank: self.rank,
            effects: effect_data,
            last_time: self.last_time,
        };

        serde_json::to_value(data).map_err(|e| e.into())
    }
}
