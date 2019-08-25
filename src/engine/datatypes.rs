#[derive(Default, Debug, Clone, Copy)]
pub struct MapPosition {
  pub x: u32,
  pub y: u32,
}

impl MapPosition {
  pub fn new(x: u32, y: u32) -> Self {
    MapPosition { x, y }
  }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct ScreenMapPosition {
  pub x: u32,
  pub y: u32,
}

impl ScreenMapPosition {
  pub fn new(x: u32, y: u32) -> Self {
    ScreenMapPosition { x, y }
  }

  pub fn from_map_position(pos: MapPosition, tile_size: SizeInPixels) -> Self {
    let x = pos.x * tile_size.width + (tile_size.width as f32 / 2.0) as u32;
    let y = pos.y * tile_size.height + (tile_size.height as f32 / 2.0) as u32;
    ScreenMapPosition { x, y }
  }

  pub fn to_map_position(&self, tile_size: SizeInPixels) -> MapPosition {
    MapPosition::new((self.x as f32 / tile_size.width as f32) as u32, (self.y as f32 / tile_size.height as f32) as u32)
  }

  pub fn centre_in_tile(&mut self, tile_size: SizeInPixels) {
    let pos = ScreenMapPosition::from_map_position(self.to_map_position(tile_size), tile_size);
    self.x = pos.x;
    self.y = pos.y;
  }
}


#[derive(Default, Debug, Clone, Copy)]
pub struct ChunkPosition {
  pub x: u32,
  pub y: u32,
}

impl ChunkPosition {
  pub fn new(x: u32, y: u32) -> Self {
    ChunkPosition { x, y }
  }

  pub fn from_map_position(pos: MapPosition, chunk_size: SizeInTiles) -> Self {
    let x = (pos.x as f32 / chunk_size.width as f32) as u32;
    let y = (pos.y as f32 / chunk_size.height as f32) as u32;
    ChunkPosition { x, y }
  }

  pub fn from_screen_map_position(pos: ScreenMapPosition, tile_size: SizeInPixels, chunk_size: SizeInTiles) -> Self {
    ChunkPosition::from_map_position(pos.to_map_position(tile_size), chunk_size)
  }
}


#[derive(Default, Debug, Clone, Copy)]
pub struct SizeInPixels {
  pub width: u32,
  pub height: u32
}

impl SizeInPixels {
  pub fn new(width: u32, height: u32) -> Self {
    SizeInPixels { width, height }
  }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct SizeInTiles {
  pub width: u32,
  pub height: u32
}

impl SizeInTiles {
  pub fn new(width: u32, height: u32) -> Self {
    SizeInTiles { width, height }
  }
}