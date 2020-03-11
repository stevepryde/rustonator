use crate::engine::world::World;
use serde::Serialize;

// Get the difference between two u32 values.
fn diffu32(a: u32, b: u32) -> u32 {
    if a > b {
        a - b
    } else {
        b - a
    }
}

#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize)]
pub struct MapPosition {
    #[serde(rename = "mapX")]
    pub x: u32,
    #[serde(rename = "mapY")]
    pub y: u32,
}

impl MapPosition {
    pub fn new(x: u32, y: u32) -> Self {
        MapPosition { x, y }
    }

    pub fn is_within_range(self, pos: MapPosition, range: u32) -> bool {
        diffu32(pos.x, self.x) < range && diffu32(pos.y, self.y) < range
    }
}

#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Serialize)]
pub struct PixelPosition<T: Serialize> {
    pub x: T,
    pub y: T,
}

impl<T: Serialize> PixelPosition<T>
where
    T: From<f64> + Into<f64> + Copy,
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
            (self.x.into() / tile_size.width as f64) as u32,
            (self.y.into() / tile_size.height as f64) as u32,
        )
    }

    pub fn centre_in_tile(&mut self, world: &World) {
        let pos = PixelPosition::from_map_position(self.to_map_position(world), world);
        self.x = pos.x;
        self.y = pos.y;
    }
}

pub type PixelPositionU32 = PixelPosition<u32>;
pub type PixelPositionF64 = PixelPosition<f64>;

#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize)]
pub struct ChunkPosition {
    #[serde(rename = "chunkX")]
    pub x: u32,
    #[serde(rename = "chunkY")]
    pub y: u32,
}

impl ChunkPosition {
    pub fn new(x: u32, y: u32) -> Self {
        ChunkPosition { x, y }
    }

    pub fn from_map_position(pos: MapPosition, world: &World) -> Self {
        let chunk_size = world.sizes().chunk_size();
        let x = (pos.x as f64 / chunk_size.width as f64) as u32;
        let y = (pos.y as f64 / chunk_size.height as f64) as u32;
        ChunkPosition { x, y }
    }

    pub fn from_pixel_position<T>(pos: PixelPosition<T>, world: &World) -> Self
    where
        T: From<f64> + Into<f64> + Serialize + Copy,
    {
        ChunkPosition::from_map_position(pos.to_map_position(world), world)
    }
}

#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize)]
pub struct SizeInPixels {
    #[serde(rename = "widthInPixels")]
    pub width: u32,
    #[serde(rename = "heightInPixels")]
    pub height: u32,
}

impl SizeInPixels {
    pub fn new(width: u32, height: u32) -> Self {
        SizeInPixels { width, height }
    }
}

#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize)]
pub struct SizeInTiles {
    #[serde(rename = "widthInTiles")]
    pub width: u32,
    #[serde(rename = "heightInTiles")]
    pub height: u32,
}

impl SizeInTiles {
    pub fn new(width: u32, height: u32) -> Self {
        SizeInTiles { width, height }
    }
}
