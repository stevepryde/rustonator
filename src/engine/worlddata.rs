use crate::{
    engine::{bomb::BombId, explosion::ExplosionId, position::MapPosition},
    error::{ZError, ZResult},
    utils::misc::Timestamp,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::convert::TryFrom;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SerWorldData(Value);

impl TryFrom<&WorldData> for SerWorldData {
    type Error = ZError;

    fn try_from(data: &WorldData) -> ZResult<Self> {
        Ok(SerWorldData(serde_json::to_value(data)?))
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct WorldData {
    data: Vec<u8>,
    width: i32,
    height: i32,
}

impl WorldData {
    pub fn new(width: i32, height: i32) -> Self {
        WorldData {
            data: vec![0; (width * height) as usize],
            width,
            height,
        }
    }

    pub fn get_index(&self, pos: MapPosition) -> Option<usize> {
        if pos.x < 0 || pos.x >= self.width || pos.y < 0 || pos.y >= self.height {
            None
        } else {
            Some(((pos.y * self.width) + pos.x) as usize)
        }
    }

    pub fn get_at(&self, pos: MapPosition) -> Option<u8> {
        self.get_index(pos).map(|index| self.data[index])
    }

    pub fn set_at(&mut self, pos: MapPosition, value: u8) {
        if let Some(index) = self.get_index(pos) {
            self.data[index] = value;
        }
    }

    pub fn get_slice(&self, index: usize, length: usize) -> &[u8] {
        &self.data[index..(index + length)]
    }

    pub fn set_slice(&mut self, index: usize, slice: &[u8]) {
        let end = index + slice.len();
        self.data.as_mut_slice()[index..end].copy_from_slice(slice);
    }

    pub fn ser(&self) -> ZResult<SerWorldData> {
        SerWorldData::try_from(self)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct WorldChunk {
    tx: i32,
    ty: i32,
    width: i32,
    height: i32,
    data: WorldData,
}

impl WorldChunk {
    pub fn new(tx: i32, ty: i32, width: i32, height: i32) -> Self {
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
pub struct InternalWorldData {
    data: Vec<InternalCellData>,
    width: i32,
    height: i32,
}

impl InternalWorldData {
    pub fn new(width: i32, height: i32) -> Self {
        InternalWorldData {
            data: vec![InternalCellData::Empty; (width * height) as usize],
            width,
            height,
        }
    }

    fn get_index(&self, pos: MapPosition) -> Option<usize> {
        if pos.x < 0 || pos.x >= self.width || pos.y < 0 || pos.y >= self.height {
            None
        } else {
            Some(((pos.y * self.width) + pos.x) as usize)
        }
    }

    pub fn get_at(&self, pos: MapPosition) -> Option<&InternalCellData> {
        self.get_index(pos).map(|index| &self.data[index])
    }

    pub fn set_at(&mut self, pos: MapPosition, value: InternalCellData) {
        if let Some(index) = self.get_index(pos) {
            self.data[index] = value;
        }
    }
}

#[derive(Debug, Clone)]
pub struct InternalMobData {
    data: Vec<Timestamp>,
    width: i32,
    height: i32,
}

impl InternalMobData {
    pub fn new(width: i32, height: i32) -> Self {
        InternalMobData {
            data: vec![Timestamp::zero(); (width * height) as usize],
            width,
            height,
        }
    }

    fn get_index(&self, pos: MapPosition) -> Option<usize> {
        if pos.x < 0 || pos.x >= self.width || pos.y < 0 || pos.y >= self.height {
            None
        } else {
            Some(((pos.y * self.width) + pos.x) as usize)
        }
    }

    pub fn get_at(&self, pos: MapPosition) -> Option<&Timestamp> {
        self.get_index(pos).map(|index| &self.data[index])
    }

    pub fn set_at(&mut self, pos: MapPosition, value: Timestamp) {
        if let Some(index) = self.get_index(pos) {
            self.data[index] = value;
        }
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
