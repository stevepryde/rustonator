use chrono::Utc;
use serde::Serialize;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Timestamp(i64);

impl Default for Timestamp {
    fn default() -> Self {
        Timestamp(Utc::now().timestamp_millis())
    }
}

impl Timestamp {
    pub fn new() -> Self {
        Timestamp::default()
    }

    pub fn zero() -> Self {
        Timestamp(0)
    }
}
