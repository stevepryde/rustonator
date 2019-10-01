use crate::{
    engine::{
        bomb::Bomb,
        config::GameConfig,
        explosion::Explosion,
        position::{MapPosition, PixelPositionF32, SizeInPixels, SizeInTiles},
        worlddata::{
            InternalCellData,
            InternalMobData,
            InternalWorldData,
            MobSpawner,
            WorldChunk,
            WorldData,
        },
        worldzone::WorldZoneData,
    },
    traits::{
        celltypes::CellType,
        worldobject::{JsonError, ToJson},
    },
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json;
use std::{collections::HashSet, convert::TryFrom};

pub struct WorldSize {
    map_size: SizeInTiles,
    tile_size: SizeInPixels,
    chunk_size: SizeInTiles,
}

impl WorldSize {
    pub fn new(width: u32, height: u32, config: &GameConfig) -> Self {
        let tile_width = 32;
        let tile_height = 32;

        // Width and height must be odd.
        let map_width = if width % 2 == 0 { width + 1 } else { width };
        let map_height = if height % 2 == 0 { height + 1 } else { height };

        let chunk_width = (config.screen_width() as f32 / tile_width as f32) as u32 + 10;
        let chunk_height = (config.screen_height() as f32 / tile_height as f32) as u32 + 10;

        WorldSize {
            map_size: SizeInTiles::new(map_width, map_height),
            tile_size: SizeInPixels::new(tile_width, tile_height),
            chunk_size: SizeInTiles::new(chunk_width, chunk_height),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSizeData {
    width: u32,
    height: u32,
    tileWidth: u32,
    tileHeight: u32,
    chunkWidth: u32,
    chunkHeight: u32,
}

impl TryFrom<serde_json::Value> for WorldSize {
    type Error = JsonError;

    fn try_from(value: serde_json::Value) -> Result<Self, JsonError> {
        let data: WorldSizeData = serde_json::from_value(value)?;

        Ok(WorldSize {
            map_size: SizeInTiles::new(data.width, data.height),
            tile_size: SizeInPixels::new(data.tileWidth, data.tileHeight),
            chunk_size: SizeInTiles::new(data.chunkWidth, data.chunkHeight),
        })
    }
}

impl ToJson for WorldSize {
    fn to_json(&self) -> Result<serde_json::Value, JsonError> {
        let data = WorldSizeData {
            width: self.map_size.width,
            height: self.map_size.height,
            tileWidth: self.tile_size.width,
            tileHeight: self.tile_size.height,
            chunkWidth: self.chunk_size.width,
            chunkHeight: self.chunk_size.height,
        };
        serde_json::to_value(data).map_err(|e| e.into())
    }
}

pub struct World {
    sizes: WorldSize,
    data: WorldData,
    data_internal: InternalWorldData,
    data_mob: InternalMobData,
    zones: WorldZoneData,
}

impl World {
    pub fn new(width: u32, height: u32, config: &GameConfig) -> Self {
        let mut world = World {
            sizes: WorldSize::new(width, height, config),
            data: WorldData::new(width, height),
            data_internal: InternalWorldData::new(width, height),
            data_mob: InternalMobData::new(width, height),
            zones: WorldZoneData::new(16, 16, width, height, 0.2),
        };

        // Create walls.
        for x in 0..world.sizes.map_size.width {
            world.set_cell(MapPosition::new(x, 0), CellType::Wall);
            world.set_cell(
                MapPosition::new(x, world.sizes.map_size.height - 1),
                CellType::Wall,
            );
        }

        for y in 0..world.sizes.map_size.height {
            world.set_cell(MapPosition::new(0, y), CellType::Wall);
            world.set_cell(
                MapPosition::new(world.sizes.map_size.width - 1, y),
                CellType::Wall,
            );

            for x in 1..((world.sizes.map_size.width as f32 / 2.0) as u32 - 2) {
                world.set_cell(MapPosition::new(x * 2, y), CellType::Wall);
            }
        }

        world
    }

    pub fn add_bomb(&mut self, bomb_id: u32, pos: MapPosition) {
        self.data_internal
            .set_at_index(self.get_index(pos), InternalCellData::Bomb(bomb_id));
    }

    pub fn add_explosion(&mut self, explosion_id: u32, pos: MapPosition) {
        self.data_internal.set_at_index(
            self.get_index(pos),
            InternalCellData::Explosion(explosion_id),
        );
    }

    fn get_index(&self, pos: MapPosition) -> usize {
        ((pos.y * self.sizes.map_size.width) + pos.x) as usize
    }

    pub fn get_cell(&self, pos: MapPosition) -> CellType {
        CellType::from(self.data.get_at_index(self.get_index(pos)))
    }

    pub fn set_cell(&mut self, pos: MapPosition, value: CellType) {
        if let CellType::Mystery = self.get_cell(pos) {
            self.zones.del_block_at_map_xy(pos);
        }
        if let CellType::Mystery = value {
            self.zones.add_block_at_map_xy(pos);
        }
        self.data.set_at_index(self.get_index(pos), value as u8);
    }

    pub fn find_nearest_blank(&self, pos: MapPosition) -> MapPosition {
        if let CellType::Empty = self.get_cell(pos) {
            return pos;
        }

        for radius in 1..20 {
            let test_length = radius as usize * 2 + 1;

            // Top.
            let startx = if radius > pos.x {
                (pos.x - radius) as usize
            } else {
                1
            };
            let endx = if startx + test_length >= self.sizes.map_size.width as usize {
                (self.sizes.map_size.width as usize - 1) - startx
            } else {
                startx + test_length
            };

            if radius < pos.y {
                let cy = pos.y - radius;
                let start_index = self.get_index(MapPosition::new(startx as u32, cy));
                for offset in 0..(endx - startx) {
                    if let CellType::Empty =
                        CellType::from(self.data.get_at_index(start_index + offset))
                    {
                        return MapPosition::new((startx + offset) as u32, cy);
                    }
                }
            }

            // Bottom.
            let cy = pos.y + radius;
            if cy < self.sizes.map_size.height - 1 {
                let start_index = self.get_index(MapPosition::new(startx as u32, cy));
                for offset in 0..(endx - startx) {
                    if let CellType::Empty =
                        CellType::from(self.data.get_at_index(start_index + offset))
                    {
                        return MapPosition::new((startx + offset) as u32, cy);
                    }
                }
            }

            // Left.
            let test_length = test_length - 2; // No need to test either end point twice.
            let starty = if radius > pos.y {
                (pos.y - radius) as usize + 1
            } else {
                1
            };
            let endy = if starty + test_length >= self.sizes.map_size.height as usize {
                (self.sizes.map_size.height as usize - 1) - starty
            } else {
                starty + test_length
            };

            if radius < pos.x {
                let cx = pos.x - radius;

                for y in starty..endy {
                    if let CellType::Empty = self.get_cell(MapPosition::new(cx, y as u32)) {
                        return MapPosition::new(cx, y as u32);
                    }
                }
            }

            // Right.
            let cx = pos.x + radius;
            if cx < self.sizes.map_size.width - 1 {
                for y in starty..endy {
                    if let CellType::Empty = self.get_cell(MapPosition::new(cx, y as u32)) {
                        return MapPosition::new(cx, y as u32);
                    }
                }
            }
        }

        MapPosition::new(1, 1)
    }

    pub fn clear_internal_cell(&mut self, pos: MapPosition) {
        let index = self.get_index(pos);
        self.data_internal
            .set_at_index(index, InternalCellData::Empty);
    }

    pub fn set_mob_data(&mut self, pos: MapPosition, timestamp: i64) {
        let index = self.get_index(pos);
        self.data_mob.set_at_index(index, timestamp);
    }

    pub fn get_mob_data(&self, pos: MapPosition) -> i64 {
        let index = self.get_index(pos);
        self.data_mob.get_at_index(index)
    }

    pub fn clear_mob_data(&mut self, pos: MapPosition) {
        let index = self.get_index(pos);
        self.data_mob.set_at_index(index, 0);
    }

    pub fn get_spawn_point(&self) -> MapPosition {
        for _ in 0..1000 {
            let tx = rand::thread_rng().gen_range(0, self.sizes.map_size.width);
            let ty = rand::thread_rng().gen_range(0, self.sizes.map_size.height);
            let pos = self.find_nearest_blank(MapPosition::new(tx, ty));

            let mut count = 0;
            if let CellType::Empty = self.get_cell(MapPosition::new(pos.x - 1, pos.y)) {
                count += 1;
            }
            if let CellType::Empty = self.get_cell(MapPosition::new(pos.x + 1, pos.y)) {
                count += 1;
            }
            if let CellType::Empty = self.get_cell(MapPosition::new(pos.x, pos.y - 1)) {
                count += 1;
            }
            if let CellType::Empty = self.get_cell(MapPosition::new(pos.x, pos.y + 1)) {
                count += 1;
            }

            if count >= 2 {
                return pos;
            }
        }

        MapPosition::new(1, 1)
    }

    pub fn get_chunk_data(&self, map_x: u32, map_y: u32) -> WorldChunk {
        let w = std::cmp::min(
            self.sizes.chunk_size.width,
            self.sizes.map_size.width - map_x,
        );
        let h = std::cmp::min(
            self.sizes.chunk_size.height,
            self.sizes.map_size.height - map_y,
        );
        let mut chunk = WorldChunk::new(map_x, map_y, w, h);
        for y in map_y..(map_y + h) {
            let index = self.get_index(MapPosition::new(map_x, y));
            chunk.set_slice(index, self.data.get_slice(index, w as usize));
        }

        chunk
    }

    pub fn is_nearby_entity(&self, pos: MapPosition, entities: &[PixelPositionF32]) -> bool {
        entities.iter().any(|p| {
            p.to_map_position(self.sizes.tile_size)
                .is_within_range(pos, 4)
        })
    }

    pub fn is_nearby_map_entity(&self, pos: MapPosition, entities: &[MapPosition]) -> bool {
        entities.iter().any(|e| e.is_within_range(pos, 4))
    }

    pub fn populate_blocks(&mut self, entities: &[PixelPositionF32], map_entities: &[MapPosition]) {
        let mut new_blocks = HashSet::new();
        for zone in self.zones.zone_iter_sorted_by_shortfall() {
            if zone.quota_reached() {
                // We're using a sorted iterator, so no point continuing.
                break;
            }

            let bx = rand::thread_rng().gen_range(0, zone.size().width) + zone.position().x;
            let by = rand::thread_rng().gen_range(0, zone.size().height) + zone.position().y;
            let blank = self.find_nearest_blank(MapPosition::new(bx, by));

            // Avoid top left corner - it's the safe space for spawning players if no blank
            // spaces were found.
            if blank.x == 1 && blank.y == 1 {
                continue;
            }

            if !self.is_nearby_entity(blank, entities)
                && !self.is_nearby_map_entity(blank, map_entities)
            {
                new_blocks.insert(blank);
                break;
            }
        }

        // Now add the blocks.
        for block in new_blocks {
            self.set_cell(block, CellType::Mystery);
        }
    }

    pub fn add_mob_spawners(&mut self) -> Vec<MobSpawner> {
        let numx = 2;
        let numy = 2;
        let stepx = self.sizes.map_size.width as f32 / numx as f32;
        let stepy = self.sizes.map_size.height as f32 / numy as f32;
        let half_stepx = stepx / 2.0;
        let half_stepy = stepy / 2.0;
        let mut mob_spawners = Vec::new();

        for py in 0..numx {
            for px in 0..numy {
                let mx = ((stepx * px as f32) + half_stepx) as u32;
                let my = ((stepy * py as f32) + half_stepy) as u32;

                let mut blank = self.find_nearest_blank(MapPosition::new(mx, my));
                if blank.x == 1 && blank.y == 1 {
                    // Try a random location.
                    let bx = rand::thread_rng().gen_range(1, self.sizes.map_size.width - 2);
                    let by = rand::thread_rng().gen_range(1, self.sizes.map_size.height - 2);
                    blank = self.find_nearest_blank(MapPosition::new(bx, by));

                    if blank.x == 1 && blank.y == 1 {
                        // Give up :(
                        continue;
                    }
                }

                // Add mob spawner.
                mob_spawners.push(MobSpawner::new(blank));
                self.set_cell(blank, CellType::MobSpawner);
            }
        }

        mob_spawners
    }
}
