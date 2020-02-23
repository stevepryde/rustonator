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
pub struct Effect {
    pub effect_type: EffectType,
    pub remaining: f64,
    pub name: String,
    pub active: bool,
}

impl Effect {
    pub fn new(effect_type: EffectType, duration: f64) -> Self {
        Effect {
            effect_type,
            remaining: duration,
            name: String::new(),
            active: true,
        }
    }

    pub fn tick(&mut self, delta_time: f64) {
        self.remaining -= delta_time;
        if self.remaining <= 0.0 {
            self.active = false;
        }
    }
}
