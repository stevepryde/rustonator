use crate::engine::position::{MapPosition, SizeInTiles};
use itertools::Itertools;
use log::*;
use std::cmp::min;

#[derive(Default, Debug, Clone, Copy)]
pub struct ZonePosition {
    pub x: i32,
    pub y: i32,
}

impl ZonePosition {
    pub fn new(x: i32, y: i32) -> Self {
        ZonePosition { x, y }
    }

    pub fn from_map_position(pos: MapPosition, zone_size: SizeInTiles) -> Self {
        let x = (pos.x as f64 / zone_size.width as f64) as i32;
        let y = (pos.y as f64 / zone_size.height as f64) as i32;
        ZonePosition { x, y }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct ZoneIndex(usize);

#[derive(Debug, Clone)]
pub struct WorldZone {
    position: MapPosition,
    size: SizeInTiles,
    num_blocks: i32,
    num_players: i32,
    block_quota: i32,
}

impl WorldZone {
    pub fn new(position: MapPosition, size: SizeInTiles, block_quota: i32) -> Self {
        WorldZone {
            position,
            size,
            num_blocks: 0,
            num_players: 0,
            block_quota,
        }
    }

    pub fn position(&self) -> MapPosition {
        self.position
    }

    pub fn size(&self) -> SizeInTiles {
        self.size
    }

    pub fn quota(&self) -> i32 {
        self.block_quota
    }

    pub fn quota_reached(&self) -> bool {
        self.num_blocks >= self.block_quota
    }
}

#[derive(Debug, Clone)]
pub struct WorldZoneData {
    zone_size: SizeInTiles,
    zones_across: usize,
    zones_down: usize,
    zones: Vec<WorldZone>,
}

impl WorldZoneData {
    pub fn new(
        zone_width: i32,
        zone_height: i32,
        width_in_tiles: i32,
        height_in_tiles: i32,
        quota_factor: f64,
    ) -> Self
    {
        let mut zones = Vec::with_capacity(
            (width_in_tiles / zone_width) as usize * (height_in_tiles / zone_height) as usize,
        );
        let mut zones_across = 0;
        let mut zones_down = 0;

        let mut zone_y = 0;
        while zone_y < height_in_tiles {
            zones_down += 1;
            let mut zone_x = 0;
            let zheight = min(height_in_tiles - zone_y, zone_height);
            while zone_x < width_in_tiles {
                if zone_y == 0 {
                    // Only count top row.
                    zones_across += 1;
                }

                let zwidth = min(width_in_tiles - zone_x, zone_width);
                zones.push(WorldZone {
                    position: MapPosition::new(zone_x, zone_y),
                    size: SizeInTiles::new(zwidth, zheight),
                    num_blocks: 0,
                    num_players: 0,
                    block_quota: ((zwidth * zheight) as f64 * quota_factor) as i32,
                });
                zone_x += zone_width;
            }
            zone_y += zone_height;
        }

        WorldZoneData {
            zone_size: SizeInTiles::new(zone_width, zone_height),
            zones_across,
            zones_down,
            zones,
        }
    }

    pub fn get_zone_index(&self, pos: ZonePosition) -> Option<ZoneIndex> {
        let index = ((pos.y * self.zones_across as i32) + pos.x) as usize;
        if index < self.zones.len() {
            Some(ZoneIndex(index))
        } else {
            None
        }
    }

    pub fn map_to_zone_index(&self, pos: MapPosition) -> Option<ZoneIndex> {
        self.get_zone_index(ZonePosition::from_map_position(pos, self.zone_size))
    }

    pub fn add_block_at_map_xy(&mut self, pos: MapPosition) {
        if let Some(zone_index) = self.map_to_zone_index(pos) {
            self.zones[zone_index.0].num_blocks += 1;
        } else {
            error!("Got invalid pos: {:?}", pos);
        }
    }

    pub fn del_block_at_map_xy(&mut self, pos: MapPosition) {
        if let Some(zone_index) = self.map_to_zone_index(pos) {
            if self.zones[zone_index.0].num_blocks > 0 {
                self.zones[zone_index.0].num_blocks -= 1;
            }
        } else {
            error!("Got invalid pos: {:?}", pos);
        }
    }

    pub fn add_player_at_map_xy(&mut self, pos: MapPosition) {
        if let Some(zone_index) = self.map_to_zone_index(pos) {
            self.zones[zone_index.0].num_players += 1;
        } else {
            error!("Got invalid pos: {:?}", pos);
        }
    }

    pub fn clear_players(&mut self) {
        for zone in &mut self.zones {
            zone.num_players = 0;
        }
    }

    pub fn zone_count(&self) -> usize {
        self.zones.len()
    }

    pub fn get_zone_at_index(&self, index: ZoneIndex) -> &WorldZone {
        &self.zones[index.0]
    }

    pub fn get_zone_at_index_mut(&mut self, index: ZoneIndex) -> &mut WorldZone {
        &mut self.zones[index.0]
    }

    pub fn get_zone_at(&self, zone_position: ZonePosition) -> Option<&WorldZone> {
        self.get_zone_index(zone_position)
            .map(|index| self.get_zone_at_index(index))
    }

    pub fn get_zone_at_mut(&mut self, zone_position: ZonePosition) -> Option<&mut WorldZone> {
        self.get_zone_index(zone_position)
            .map(move |index| self.get_zone_at_index_mut(index))
    }

    pub fn zone_iter(&self) -> impl Iterator<Item = &WorldZone> {
        self.zones.iter()
    }

    /// Return iterator providing WorldZone objects sorted by (quota -
    /// num_blocks) descending.
    pub fn zone_iter_sorted_by_shortfall(&self) -> impl Iterator<Item = &WorldZone> {
        self.zones.iter().sorted_by(|a, b| {
            Ord::cmp(
                &(b.block_quota - b.num_blocks),
                &(a.block_quota - a.num_blocks),
            )
        })
    }
}
