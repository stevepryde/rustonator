use crate::{
    engine::{
        player::{Player, PlayerId},
        position::MapPosition,
    },
    tools::itemstore::HasId,
    utils::misc::Timestamp,
};
use serde::{Deserialize, Serialize};
use std::{
    ops::{Add, AddAssign, Deref, SubAssign},
    time::Duration,
};

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BombId(u64);

impl From<u64> for BombId {
    fn from(value: u64) -> Self {
        BombId(value)
    }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct BombRange(u32);

impl From<u32> for BombRange {
    fn from(value: u32) -> Self {
        BombRange(value)
    }
}

impl Deref for BombRange {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Add<u32> for BombRange {
    type Output = BombRange;

    fn add(self, rhs: u32) -> Self::Output {
        BombRange(self.0 + rhs)
    }
}

impl AddAssign<u32> for BombRange {
    fn add_assign(&mut self, rhs: u32) {
        self.0 += rhs;
    }
}

impl SubAssign<u32> for BombRange {
    fn sub_assign(&mut self, rhs: u32) {
        self.0 -= rhs;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct BombTime(f64);

impl BombTime {
    pub fn millis(self) -> f64 {
        self.0
    }

    pub fn clear(&mut self) {
        self.0 = 0.0;
    }

    pub fn is_done(self) -> bool {
        self.0 <= 0.0
    }
}

impl From<f64> for BombTime {
    fn from(value: f64) -> Self {
        BombTime(value)
    }
}

impl Deref for BombTime {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Add<f64> for BombTime {
    type Output = BombTime;

    fn add(self, rhs: f64) -> Self::Output {
        BombTime(self.0 + rhs)
    }
}

impl AddAssign<f64> for BombTime {
    fn add_assign(&mut self, rhs: f64) {
        self.0 += rhs;
    }
}

impl SubAssign<f64> for BombTime {
    fn sub_assign(&mut self, rhs: f64) {
        self.0 -= rhs;
    }
}

impl Add<Duration> for BombTime {
    type Output = BombTime;

    fn add(self, rhs: Duration) -> Self::Output {
        BombTime(self.0 + rhs.as_secs_f64())
    }
}

impl AddAssign<Duration> for BombTime {
    fn add_assign(&mut self, rhs: Duration) {
        self.0 += rhs.as_secs_f64();
    }
}

impl SubAssign<Duration> for BombTime {
    fn sub_assign(&mut self, rhs: Duration) {
        self.0 -= rhs.as_secs_f64();
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Bomb {
    id: BombId,
    pid: PlayerId,
    pname: String,
    active: bool,
    #[serde(flatten)]
    position: MapPosition,
    remaining: BombTime,
    range: BombRange,
    timestamp: Timestamp,
}

impl Bomb {
    pub fn new(player: &Player, position: MapPosition) -> Self {
        Bomb {
            id: BombId::from(0),
            pid: player.id(),
            pname: player.name().to_owned(),
            active: true,
            position,
            remaining: player.bomb_time(),
            range: player.range(),
            // Set the timestamp to the explosion timestamp
            timestamp: Timestamp::new() + player.bomb_time(),
        }
    }

    pub fn id(&self) -> BombId {
        self.id
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn pid(&self) -> PlayerId {
        self.pid
    }

    pub fn pname(&self) -> &str {
        self.pname.as_str()
    }

    pub fn position(&self) -> MapPosition {
        self.position
    }

    pub fn range(&self) -> BombRange {
        self.range
    }

    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }

    pub fn tick(&mut self, delta_time: f64) -> bool {
        self.remaining -= delta_time;
        if self.remaining.is_done() {
            self.remaining.clear();
            self.active = false;
            true
        } else {
            false
        }
    }

    pub fn terminate(&mut self) {
        self.active = false;
    }
}

impl HasId<BombId> for Bomb {
    fn set_id(&mut self, id: BombId) {
        self.id = id;
    }
}
