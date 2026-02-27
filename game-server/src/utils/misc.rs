use crate::engine::bomb::BombTime;
use serde::Serialize;
use std::{
    ops::Add,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct Timestamp(i64);

impl Default for Timestamp {
    fn default() -> Self {
        Timestamp(now_millis())
    }
}

impl Timestamp {
    pub fn new() -> Self {
        Timestamp::default()
    }

    pub fn zero() -> Self {
        Timestamp(0)
    }

    pub fn is_zero(self) -> bool {
        self.0 == 0
    }

    pub fn is_past(self) -> bool {
        // Allow a 1 second buffer.
        self.0 == 0 || self.0 < now_millis() - 1000
    }
}

impl Add<Duration> for Timestamp {
    type Output = Timestamp;

    fn add(self, rhs: Duration) -> Self::Output {
        Timestamp(self.0 + rhs.as_millis() as i64)
    }
}

impl Add<BombTime> for Timestamp {
    type Output = Timestamp;

    fn add(self, rhs: BombTime) -> Self::Output {
        Timestamp(self.0 + rhs.millis() as i64)
    }
}
