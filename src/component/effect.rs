use crate::traits::{
    randenum::RandEnumFrom,
    worldobject::{JsonError, ToJson},
};
use serde::{Deserialize, Serialize};
use serde_json;
use std::convert::TryFrom;

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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EffectData {
    effect_type: u8,
    remaining: f32,
    name: String,
    active: bool,
}

impl TryFrom<serde_json::Value> for Effect {
    type Error = JsonError;

    fn try_from(value: serde_json::Value) -> Result<Self, JsonError> {
        let data: EffectData = serde_json::from_value(value)?;

        Ok(Effect {
            active: data.active,
            name: data.name,
            remaining: data.remaining,
            effect_type: EffectType::from(data.effect_type),
        })
    }
}

impl ToJson for Effect {
    fn to_json(&self) -> Result<serde_json::Value, JsonError> {
        let data = EffectData {
            effect_type: self.effect_type as u8,
            remaining: self.remaining,
            name: self.name.clone(),
            active: self.active,
        };
        serde_json::to_value(data).map_err(|e| e.into())
    }
}
