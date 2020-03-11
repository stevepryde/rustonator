use crate::engine::bomb::BombId;
use crate::engine::explosion::ExplosionId;
use crate::engine::position::MapPosition;
use crate::error::{ZError, ZResult};
use crate::utils::misc::Timestamp;
use serde::Serialize;
use serde_json::Value;
use std::convert::TryFrom;

#[derive(Debug, Serialize)]
#[serde(transparent)]
pub struct SerWorldData(Value);

impl TryFrom<&WorldData> for SerWorldData {
    type Error = ZError;

    fn try_from(data: &WorldData) -> ZResult<Self> {
        Ok(SerWorldData(serde_json::to_value(data)?))
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct WorldData(Vec<u8>);

impl WorldData {
    pub fn new(width: u32, height: u32) -> Self {
        WorldData(vec![0; (width * height) as usize])
    }

    pub fn get_at_index(&self, index: usize) -> Option<u8> {
        if index >= self.0.len() {
            None
        } else {
            Some(self.0[index])
        }
    }

    pub fn set_at_index(&mut self, index: usize, value: u8) {
        // TODO: fail on bad index?
        if index < self.0.len() {
            self.0[index] = value;
        }
    }

    pub fn get_slice(&self, index: usize, length: usize) -> &[u8] {
        &self.0[index..(index + length)]
    }

    pub fn set_slice(&mut self, index: usize, slice: &[u8]) {
        let end = index + slice.len();
        self.0.as_mut_slice()[index..end].copy_from_slice(slice);
    }

    pub fn ser(&self) -> ZResult<SerWorldData> {
        SerWorldData::try_from(self)
    }
}

#[derive(Debug, Clone)]
pub struct WorldChunk {
    tx: u32,
    ty: u32,
    width: u32,
    height: u32,
    data: WorldData,
}

impl WorldChunk {
    pub fn new(tx: u32, ty: u32, width: u32, height: u32) -> Self {
        WorldChunk {
            tx,
            ty,
            width,
            height,
            data: WorldData::new(width, height),
        }
    }

    pub fn set_slice(&mut self, index: usize, slice: &[u8]) {
        self.data.set_slice(index, slice);
    }
}

#[derive(Debug, Clone)]
pub enum InternalCellData {
    Empty,
    Bomb(BombId),
    Explosion(ExplosionId),
}

#[derive(Debug, Clone)]
pub struct InternalWorldData(Vec<InternalCellData>);

impl InternalWorldData {
    pub fn new(width: u32, height: u32) -> Self {
        InternalWorldData(vec![InternalCellData::Empty; (width * height) as usize])
    }

    pub fn get_at_index(&self, index: usize) -> &InternalCellData {
        // TODO: do we need a range check here?
        &self.0[index]
    }

    pub fn set_at_index(&mut self, index: usize, value: InternalCellData) {
        // TODO: do we need a range check here?
        self.0[index] = value;
    }
}

#[derive(Debug, Clone)]
pub struct InternalMobData(Vec<Timestamp>);

impl InternalMobData {
    pub fn new(width: u32, height: u32) -> Self {
        InternalMobData(vec![Timestamp::zero(); (width * height) as usize])
    }

    pub fn get_at_index(&self, index: usize) -> Timestamp {
        // TODO: do we need a range check here?
        self.0[index]
    }

    pub fn set_at_index(&mut self, index: usize, value: Timestamp) {
        // TODO: do we need a range check here?
        self.0[index] = value;
    }
}

#[derive(Debug, Clone)]
pub struct MobSpawner {
    position: MapPosition,
}

impl MobSpawner {
    pub fn new(position: MapPosition) -> Self {
        MobSpawner { position }
    }

    pub fn position(&self) -> MapPosition {
        self.position
    }
}
