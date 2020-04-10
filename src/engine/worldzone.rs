use crate::engine::position::{MapPosition, SizeInTiles};
use itertools::Itertools;

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
        let mut zones_across = (width_in_tiles as f64 / zone_width as f64) as usize;
        let mut last_width = zone_width;
        let remainder_width = width_in_tiles as usize % zones_across;
        if remainder_width > 0 {
            zones_across += 1;
            last_width = remainder_width as i32;
        }

        let mut zones_down = (height_in_tiles as f64 / zone_height as f64) as usize;
        let mut last_height = zone_height;
        let remainder_height = height_in_tiles as usize % zones_down;
        if remainder_height > 0 {
            zones_down += 1;
            last_height = remainder_height as i32;
        }

        let mut zones = Vec::with_capacity(zones_across * zones_down);
        for y in 0..zones_down {
            for x in 0..zones_across {
                let zwidth = if x < zones_across - 1 {
                    zone_width
                } else {
                    last_width
                };
                let zheight = if y < zones_down - 1 {
                    zone_height
                } else {
                    last_height
                };

                zones.push(WorldZone {
                    position: MapPosition::new(x as i32 * zone_width, y as i32 * zone_height),
                    size: SizeInTiles::new(zwidth, zheight),
                    num_blocks: 0,
                    num_players: 0,
                    block_quota: ((zwidth * zheight) as f64 * quota_factor) as i32,
                });
            }
        }

        WorldZoneData {
            zone_size: SizeInTiles::new(zone_width, zone_height),
            zones_across,
            zones_down,
            zones,
        }
    }

    pub fn get_zone_index(&self, pos: ZonePosition) -> ZoneIndex {
        ZoneIndex(((pos.y * self.zones_across as i32) + pos.x) as usize)
    }

    pub fn map_to_zone_index(&self, pos: MapPosition) -> ZoneIndex {
        self.get_zone_index(ZonePosition::from_map_position(pos, self.zone_size))
    }

    pub fn add_block_at_map_xy(&mut self, pos: MapPosition) {
        let zone_index = self.map_to_zone_index(pos);
        self.zones[zone_index.0].num_blocks += 1;
    }

    pub fn del_block_at_map_xy(&mut self, pos: MapPosition) {
        let zone_index = self.map_to_zone_index(pos);

        if self.zones[zone_index.0].num_blocks > 0 {
            self.zones[zone_index.0].num_blocks -= 1;
        }
    }

    pub fn add_player_at_map_xy(&mut self, pos: MapPosition) {
        let zone_index = self.map_to_zone_index(pos);
        self.zones[zone_index.0].num_players += 1;
    }

    pub fn clear_players(&mut self) {
        for zone in &mut self.zones {
            zone.num_players = 0;
        }
    }

    pub fn get_zone_at_index(&self, index: ZoneIndex) -> &WorldZone {
        &self.zones[index.0]
    }

    pub fn get_zone_at_index_mut(&mut self, index: ZoneIndex) -> &mut WorldZone {
        &mut self.zones[index.0]
    }

    pub fn get_zone_at(&self, zone_position: ZonePosition) -> &WorldZone {
        let index = self.get_zone_index(zone_position);
        self.get_zone_at_index(index)
    }

    pub fn get_zone_at_mut(&mut self, zone_position: ZonePosition) -> &mut WorldZone {
        let index = self.get_zone_index(zone_position);
        self.get_zone_at_index_mut(index)
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
