use crate::traits::jsonobject::{JSONObject, JSONValue};
use crate::traits::randenum::RandEnumFrom;
use rand::Rng;
use serde_json::json;

#[derive(Copy, Clone, Debug)]
pub enum EffectType {
    SpeedUp = 0,
    SlowDown = 1,
    Invincibility = 2,
}

impl From<u8> for EffectType {
    fn from(value: u8) -> Self {
        match value {
            0 => EffectType::SpeedUp,
            1 => EffectType::SlowDown,
            2 => EffectType::Invincibility,
            _ => panic!("Invalid effect type: {}", value),
        }
    }
}

impl RandEnumFrom<u8> for EffectType {
    fn get_enum_values() -> Vec<u8> {
        (0..3).collect()
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

impl JSONObject for Effect {
    fn to_json(&self) -> serde_json::Value {
        json!({
            "type": self.effect_type as u8,
            "remaining": self.remaining,
            "name": self.name,
            "active": self.active
        })
    }

    fn from_json(&mut self, data: &serde_json::Value) {
        let sv = JSONValue::new(data);
        self.effect_type = EffectType::from(sv.get_u32("type") as u8);
        self.remaining = sv.get_f32("remaining");
        self.name = sv.get_string("name");
        self.active = sv.get_bool("active");
    }
}
