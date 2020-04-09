use crate::engine::world::World;
use itertools::Chunk;
use rand::{thread_rng, Rng};
use serde::Serialize;
use std::ops::{Add, Mul};

// Get the difference between two i32 values.
fn diffu32(a: i32, b: i32) -> i32 {
    if a > b {
        a - b
    } else {
        b - a
    }
}

#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct PositionOffset {
    pub x: i32,
    pub y: i32,
}

impl PositionOffset {
    pub fn new(x: i32, y: i32) -> Self {
        PositionOffset { x, y }
    }

    pub fn up(dist: i32) -> Self {
        PositionOffset { x: 0, y: -dist }
    }

    pub fn down(dist: i32) -> Self {
        PositionOffset { x: 0, y: dist }
    }

    pub fn left(dist: i32) -> Self {
        PositionOffset { x: -dist, y: 0 }
    }

    pub fn right(dist: i32) -> Self {
        PositionOffset { x: dist, y: 0 }
    }
}

impl Mul<i32> for PositionOffset {
    type Output = Self;

    fn mul(self, other: i32) -> Self {
        PositionOffset {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl Add<PositionOffset> for PositionOffset {
    type Output = Self;

    fn add(self, other: PositionOffset) -> Self {
        PositionOffset {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize)]
pub struct MapPosition {
    #[serde(rename = "mapX")]
    pub x: i32,
    #[serde(rename = "mapY")]
    pub y: i32,
}

impl MapPosition {
    pub fn new(x: i32, y: i32) -> Self {
        MapPosition { x, y }
    }

    pub fn is_within_range(self, pos: MapPosition, range: i32) -> bool {
        diffu32(pos.x, self.x) < range && diffu32(pos.y, self.y) < range
    }

    pub fn is_top_left(self) -> bool {
        self.x == 1 && self.y == 1
    }

    pub fn up(self, dist: i32) -> Self {
        Self {
            x: self.x,
            y: self.y - dist,
        }
    }
    pub fn down(self, dist: i32) -> Self {
        Self {
            x: self.x,
            y: self.y + dist,
        }
    }
    pub fn left(self, dist: i32) -> Self {
        Self {
            x: self.x - dist,
            y: self.y,
        }
    }
    pub fn right(self, dist: i32) -> Self {
        Self {
            x: self.x + dist,
            y: self.y,
        }
    }

    pub fn random_offset(self, range: u32) -> Self {
        let irange = range as i32; // Don't worry, the range will always be small.
        Self {
            x: self.x + thread_rng().gen_range(-irange, irange),
            y: self.y + thread_rng().gen_range(-irange, irange),
        }
    }

    pub fn distance_to(self, pos: MapPosition) -> u32 {
        (pos.y - self.y).abs() as u32 + (pos.x - self.x).abs() as u32
    }
}

impl Add<PositionOffset> for MapPosition {
    type Output = Self;
    fn add(self, other: PositionOffset) -> Self {
        MapPosition {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Serialize)]
pub struct PixelPosition<T: Serialize> {
    pub x: T,
    pub y: T,
}

impl<T: Serialize> PixelPosition<T>
where T: From<f64> + Into<f64> + Copy
{
    pub fn new(x: T, y: T) -> Self {
        PixelPosition { x, y }
    }

    pub fn from_map_position(pos: MapPosition, world: &World) -> Self {
        let tile_size = world.sizes().tile_size();
        let x = T::from(pos.x as f64 * tile_size.width as f64 + (tile_size.width as f64 / 2.0));
        let y = T::from(pos.y as f64 * tile_size.height as f64 + (tile_size.height as f64 / 2.0));
        Self::new(x, y)
    }

    pub fn to_map_position(&self, world: &World) -> MapPosition {
        let tile_size = world.sizes().tile_size();
        MapPosition::new(
            (self.x.into() / tile_size.width as f64) as i32,
            (self.y.into() / tile_size.height as f64) as i32,
        )
    }

    pub fn centre_in_tile(&mut self, world: &World) {
        let pos = PixelPosition::from_map_position(self.to_map_position(world), world);
        self.x = pos.x;
        self.y = pos.y;
    }
}

pub type PixelPositionU32 = PixelPosition<i32>;
pub type PixelPositionF64 = PixelPosition<f64>;

#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize)]
pub struct ChunkPosition {
    #[serde(rename = "chunkX")]
    pub x: i32,
    #[serde(rename = "chunkY")]
    pub y: i32,
}

impl ChunkPosition {
    pub fn new(x: i32, y: i32) -> Self {
        ChunkPosition { x, y }
    }

    pub fn from_map_position(pos: MapPosition, world: &World) -> Self {
        let chunk_size = world.sizes().chunk_size();
        let x = (pos.x as f64 / chunk_size.width as f64) as i32;
        let y = (pos.y as f64 / chunk_size.height as f64) as i32;
        ChunkPosition { x, y }
    }

    pub fn from_pixel_position<T>(pos: PixelPosition<T>, world: &World) -> Self
    where T: From<f64> + Into<f64> + Serialize + Copy {
        ChunkPosition::from_map_position(pos.to_map_position(world), world)
    }

    pub fn up(self, dist: i32) -> Self {
        Self {
            x: self.x,
            y: self.y - dist,
        }
    }
    pub fn down(self, dist: i32) -> Self {
        Self {
            x: self.x,
            y: self.y + dist,
        }
    }
    pub fn left(self, dist: i32) -> Self {
        Self {
            x: self.x - dist,
            y: self.y,
        }
    }
    pub fn right(self, dist: i32) -> Self {
        Self {
            x: self.x + dist,
            y: self.y,
        }
    }
}

impl Add<PositionOffset> for ChunkPosition {
    type Output = Self;
    fn add(self, other: PositionOffset) -> Self {
        ChunkPosition {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize)]
pub struct SizeInPixels {
    #[serde(rename = "widthInPixels")]
    pub width: i32,
    #[serde(rename = "heightInPixels")]
    pub height: i32,
}

impl SizeInPixels {
    pub fn new(width: i32, height: i32) -> Self {
        SizeInPixels { width, height }
    }
}

#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize)]
pub struct SizeInTiles {
    #[serde(rename = "widthInTiles")]
    pub width: i32,
    #[serde(rename = "heightInTiles")]
    pub height: i32,
}

impl SizeInTiles {
    pub fn new(width: i32, height: i32) -> Self {
        SizeInTiles { width, height }
    }
}
