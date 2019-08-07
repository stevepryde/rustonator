use crate::component::action::Action;
use crate::component::effect::{Effect, EffectType};
use crate::traits::gameobject::{GameObject, SuperValue};
use bitflags::bitflags;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::json;

bitflags! {
    #[derive(Default, Serialize, Deserialize)]
    pub struct PlayerFlags: u32 {
        const WALK_THROUGH_BOMBS = 0b0001;
        const INVINCIBLE         = 0b0010;
    }
}

pub struct Player {
    id: String,
    active: bool,
    x: f32,
    y: f32,
    action: Action,
    speed: f32,
    image: String,
    range: u8,
    bomb_time: f32,
    max_bombs: u32,
    cur_bombs: u32,
    flags: PlayerFlags,
    score: u32,
    name: String,
    rank: u32,
    effects: [Vec<Effect>; 2],
    effect_index: usize,
    last_time: f32,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            id: String::new(),
            active: true,
            x: 0.0,
            y: 0.0,
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

    pub fn update(&mut self, delta_time: f32) {
        let action = self.action.clone();
        self.update_with_temp_action(&action, delta_time);
    }

    fn update_with_temp_action(&mut self, tmp_action: &Action, delta_time: f32) {
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
        self.x += tmp_action.x * delta_time * effective_speed;
        self.y += tmp_action.y * delta_time * effective_speed;
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

    fn setxy(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    fn set_action(&mut self, action: Action) {
        self.action = action;
    }

    fn add_random_effect(&mut self) -> String {
        let effect = Effect::new(
            EffectType::random(),
            rand::thread_rng().gen_range(3.0f32, 8.0f32),
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

impl GameObject for Player {
    fn to_json(&self) -> serde_json::Value {
        let mut effect_data = Vec::new();
        for effect in &self.effects[self.effect_index] {
            effect_data.push(effect.to_json());
        }

        json!({
            "id": self.id,
            "active": self.active,
            "x": self.x,
            "y": self.y,
            "action": self.action.to_json(),
            "speed": self.speed,
            "image": self.image,
            "range": self.range,
            "bombTime": self.bomb_time,
            "maxBombs": self.max_bombs,
            "curBombs": self.cur_bombs,
            "flags": self.flags,
            "score": self.score,
            "name": self.name,
            "rank": self.rank,
            "effects": effect_data
        })
    }
    fn from_json(&mut self, data: &serde_json::Value) {
        let sv = SuperValue::new(data);
        self.id = sv.get_string("id");
        self.active = sv.get_bool("active");
        self.x = sv.get_f32("x");
        self.y = sv.get_f32("y");
        self.action.from_json(sv.get_value("action"));
        self.speed = sv.get_f32("speed");
        self.image = sv.get_string("image");
        self.range = sv.get_u32("range") as u8;
        self.bomb_time = sv.get_f32("bombTime");
        self.max_bombs = sv.get_u32("maxBombs");
        self.cur_bombs = sv.get_u32("curBombs");
        self.flags = PlayerFlags {
            bits: sv.get_u32("flags"),
        };
        self.score = sv.get_u32("score");
        self.name = sv.get_string("name");
        self.rank = sv.get_u32("rank");

        // TODO: reuse effects if possible.
        self.effects[self.effect_index].clear();
        for effect_data in sv.get_vec("effects") {
            let mut effect = Effect::new(EffectType::SpeedUp, 0.0);
            effect.from_json(&effect_data);
            self.effects[self.effect_index].push(effect);
        }
    }
}
