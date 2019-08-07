use crate::traits::gameobject::{GameObject, SuperValue};
use serde_json::json;
use rand::Rng;

#[derive(Copy, Clone, Debug)]
pub enum EffectType {
    SpeedUp = 0,
    SlowDown = 1,
    Invincibility = 2,
}

impl EffectType {
    pub fn from(value: u8) -> Self {
        match value {
            0 => EffectType::SpeedUp,
            1 => EffectType::SlowDown,
            2 => EffectType::Invincibility,
            _ => panic!("Invalid effect type: {}", value),
        }
    }

    pub fn random() -> Self {
        EffectType::from(rand::thread_rng().gen_range(0, 3))
    }
}

#[derive(Debug)]
pub struct Effect {
    pub effect_type: EffectType,
    pub remaining: f32,
    pub name: String,
    pub active: bool,
}

impl Effect {
    pub fn new(effect_type: EffectType, duration: f32) -> Self {
        Effect {
            effect_type,
            remaining: duration,
            name: String::new(),
            active: true,
        }
    }

    pub fn tick(&mut self, delta_time: f32) {
        self.remaining -= delta_time;
        if self.remaining <= 0.0 {
            self.active = false;
        }
    }
}

impl GameObject for Effect {
    fn to_json(&self) -> serde_json::Value {
        json!({
            "type": self.effect_type as u8,
            "remaining": self.remaining,
            "name": self.name,
            "active": self.active
        })
    }

    fn from_json(&mut self, data: &serde_json::Value) {
        let sv = SuperValue::new(data);
        self.effect_type = EffectType::from(sv.get_u32("type") as u8);
        self.remaining = sv.get_f32("remaining");
        self.name = sv.get_string("name");
        self.active = sv.get_bool("active");
    }
}
