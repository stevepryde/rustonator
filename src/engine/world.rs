use crate::engine::bomb::Bomb;
use crate::engine::config::GameConfig;
use crate::engine::explosion::Explosion;
use crate::engine::mob::Mob;
use crate::engine::player::Player;
use crate::engine::worlddata::{InternalMobData, InternalWorldData, MobSpawner, WorldData};
use crate::engine::worldzone::WorldZoneData;
use crate::traits::celltypes::{CanPass, CellType};
use crate::traits::jsonobject::{JSONObject, JSONValue};
use serde_json::json;
use std::collections::HashMap;

pub struct World {
    width_in_tiles: u32,
    height_in_tiles: u32,
    tile_width: u32,
    tile_height: u32,
    chunk_width: u32,
    chunk_height: u32,
    data: WorldData,
    data_internal: InternalWorldData,
    data_mob: InternalMobData,
    zones: WorldZoneData,
    mob_spawners: Vec<MobSpawner>,
    players: HashMap<String, Player>,
    mobs: HashMap<u32, Mob>,
    bombs: HashMap<u32, Bomb>,
    explosions: HashMap<u32, Explosion>,
}

impl World {
    pub fn new(width: u32, height: u32, config: &GameConfig) -> Self {
        let tile_width = 32;
        let tile_height = 32;

        World {
            width_in_tiles: width,
            height_in_tiles: height,
            tile_width,
            tile_height,
            chunk_width: (config.get_width() as f32 / tile_width as f32) as u32 + 10,
            chunk_height: (config.get_height() as f32 / tile_height as f32) as u32 + 10,
            data: WorldData::new(width, height),
            data_internal: InternalWorldData::new(width, height),
            data_mob: InternalMobData::new(width, height),
            zones: WorldZoneData::new(16, 16, width, height, 0.2),
            mob_spawners: Vec::with_capacity(4),
            players: HashMap::new(),
            mobs: HashMap::new(),
            bombs: HashMap::new(),
            explosions: HashMap::new(),
        }
    }

    fn get_index(&self, x: u32, y: u32) -> usize {
        ((y * self.width_in_tiles) + x) as usize
    }

    pub fn get_cell(&self, x: u32, y: u32) -> CellType {
        CellType::from(self.data.get_at_index(self.get_index(x, y)))
    }

    pub fn set_cell(&mut self, x: u32, y: u32, value: CellType) {
        self.data.set_at_index(self.get_index(x, y), value as u8);
    }

    pub fn to_screen_x(&self, mx: u32) -> u32 {
        mx * self.tile_width + (self.tile_width as f32 / 2.0) as u32
    }

    pub fn to_screen_y(&self, my: u32) -> u32 {
        my * self.tile_height + (self.tile_height as f32 / 2.0) as u32
    }

    pub fn to_map_x(&self, sx: u32) -> u32 {
        (sx as f32 / self.tile_width as f32) as u32
    }

    pub fn to_map_y(&self, sy: u32) -> u32 {
        (sy as f32 / self.tile_height as f32) as u32
    }

    pub fn fix_screen_x(&self, sx: u32) -> u32 {
        self.to_screen_x(self.to_map_x(sx))
    }

    pub fn fix_screen_y(&self, sy: u32) -> u32 {
        self.to_screen_y(self.to_map_y(sy))
    }

    pub fn map_to_chunk_x(&self, mx: u32) -> u32 {
        (mx as f32 / self.chunk_width as f32) as u32
    }

    pub fn map_to_chunk_y(&self, my: u32) -> u32 {
        (my as f32 / self.chunk_height as f32) as u32
    }

    pub fn screen_to_chunk_x(&self, sx: u32) -> u32 {
        self.map_to_chunk_x(self.to_map_x(sx))
    }

    pub fn screen_to_chunk_y(&self, sy: u32) -> u32 {
        self.map_to_chunk_y(self.to_map_y(sy))
    }

    pub fn find_nearest_blank(&self, mx: u32, my: u32) -> (u32, u32) {
        if let CellType::Empty = self.get_cell(mx, my) {
            return (mx, my);
        }

        for radius in 1..20 {
            let test_length = radius as usize * 2 + 1;

            // Top.
            let startx = if radius > mx {
                (mx - radius) as usize
            } else {
                1
            };
            let endx = if startx + test_length >= self.width_in_tiles as usize {
                (self.width_in_tiles as usize - 1) - startx
            } else {
                startx + test_length
            };

            if radius < my {
                let cy = my - radius;
                let start_index = self.get_index(startx as u32, cy);
                for offset in 0..(endx - startx) {
                    if let CellType::Empty =
                        CellType::from(self.data.get_at_index(start_index + offset))
                    {
                        return ((startx + offset) as u32, cy);
                    }
                }
            }

            // Bottom.
            let cy = my + radius;
            if cy < self.height_in_tiles - 1 {
                let start_index = self.get_index(startx as u32, cy);
                for offset in 0..(endx - startx) {
                    if let CellType::Empty =
                        CellType::from(self.data.get_at_index(start_index + offset))
                    {
                        return ((startx + offset) as u32, cy);
                    }
                }
            }

            // Left.
            let test_length = test_length - 2; // No need to test either end point twice.
            let starty = if radius > my {
                (my - radius) as usize + 1
            } else {
                1
            };
            let endy = if starty + test_length >= self.height_in_tiles as usize {
                (self.height_in_tiles as usize - 1) - starty
            } else {
                starty + test_length
            };

            if radius < mx {
                let cx = mx - radius;

                for y in starty..endy {
                    if let CellType::Empty = self.get_cell(cx, y as u32) {
                        return (cx, y as u32);
                    }
                }
            }

            // Right.
            let cx = mx + radius;
            if cx < self.width_in_tiles - 1 {
                for y in starty..endy {
                    if let CellType::Empty = self.get_cell(cx, y as u32) {
                        return (cx, y as u32);
                    }
                }
            }
        }

        (1, 1)
    }

    // pub fn setBomb
}

impl JSONObject for World {
    fn to_json(&self) -> serde_json::Value {
        json!({
            "width": self.width_in_tiles,
            "height": self.height_in_tiles,
            "tileWidth": self.tile_width,
            "tileHeight": self.tile_height,
            "chunkWidth": self.chunk_width,
            "chunkHeight": self.chunk_height
        })
    }

    fn from_json(&mut self, data: &serde_json::Value) {
        let sv = JSONValue::new(data);
        self.width_in_tiles = sv.get_u32("width");
        self.height_in_tiles = sv.get_u32("height");
        self.tile_width = sv.get_u32("tileWidth");
        self.tile_height = sv.get_u32("tileHeight");
        self.chunk_width = sv.get_u32("chunkWidth");
        self.chunk_height = sv.get_u32("chunkHeight");
    }
}
