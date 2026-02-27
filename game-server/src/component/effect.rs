use crate::traits::randenum::RandEnumFrom;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Copy, Clone, Debug, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum EffectType {
    SpeedUp = 0,
    SlowDown = 1,
    Invincibility = 2,
}

impl EffectType {
    pub fn name(self) -> String {
        match self {
            EffectType::SpeedUp => String::from(">>"),
            EffectType::SlowDown => String::from("<<"),
            EffectType::Invincibility => String::from("∞"),
        }
    }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Effect {
    pub effect_type: EffectType,
    pub remaining: f64,
    pub active: bool,
}

impl Effect {
    pub fn new(effect_type: EffectType, duration: f64) -> Self {
        Effect {
            effect_type,
            remaining: duration,
            active: true,
        }
    }

    pub fn name(&self) -> String {
        self.effect_type.name()
    }

    pub fn tick(&mut self, delta_time: f64) {
        self.remaining -= delta_time;
        if self.remaining <= 0.0 {
            self.active = false;
        }
    }
}
