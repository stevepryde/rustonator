#[derive(Debug, Clone)]
pub struct WorldZone {
    width: u32,
    height: u32,
    num_blocks: u32,
    num_players: u32,
    block_quota: u32,
}

impl WorldZone {
    pub fn new(width: u32, height: u32, block_quota: u32) -> Self {
        WorldZone {
            width,
            height,
            num_blocks: 0,
            num_players: 0,
            block_quota,
        }
    }

    pub fn quota_reached(&self) -> bool {
        self.num_blocks >= self.block_quota
    }
}

#[derive(Debug, Clone)]
pub struct WorldZoneData {
    zone_width: u32, // Width & height units are the number of tiles.
    zone_height: u32,
    zones_across: usize,
    zones_down: usize,
    zones: Vec<WorldZone>,
}

impl WorldZoneData {
    pub fn new(
        zone_width: u32,
        zone_height: u32,
        width_in_tiles: u32,
        height_in_tiles: u32,
        quota_factor: f32,
    ) -> Self {
        let mut zones_across = (width_in_tiles as f32 / zone_width as f32) as usize;
        let mut last_width = zone_width;
        let remainder_width = width_in_tiles as usize % zones_across;
        if remainder_width > 0 {
            zones_across += 1;
            last_width = remainder_width as u32;
        }

        let mut zones_down = (height_in_tiles as f32 / zone_height as f32) as usize;
        let mut last_height = zone_height;
        let remainder_height = height_in_tiles as usize % zones_down;
        if remainder_height > 0 {
            zones_down += 1;
            last_height = remainder_height as u32;
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
                    width: zwidth,
                    height: zheight,
                    num_blocks: 0,
                    num_players: 0,
                    block_quota: ((zwidth * zheight) as f32 * quota_factor) as u32,
                });
            }
        }

        WorldZoneData {
            zone_width,
            zone_height,
            zones_across,
            zones_down,
            zones,
        }
    }

    pub fn get_zone_index(&self, zone_x: u32, zone_y: u32) -> usize {
        ((zone_y * self.zones_across as u32) + zone_x) as usize
    }

    pub fn map_to_zone_x(&self, mx: u32) -> u32 {
        (mx as f32 / self.zone_width as f32) as u32
    }

    pub fn map_to_zone_y(&self, my: u32) -> u32 {
        (my as f32 / self.zone_height as f32) as u32
    }

    pub fn map_to_zone_index(&self, mx: u32, my: u32) -> usize {
        let zx = self.map_to_zone_x(mx);
        let zy = self.map_to_zone_y(my);
        self.get_zone_index(zx, zy)
    }

    pub fn add_block_at_map_xy(&mut self, mx: u32, my: u32) {
        let zone_index = self.map_to_zone_index(mx, my);
        self.zones[zone_index].num_blocks += 1;
    }

    pub fn del_block_at_map_xy(&mut self, mx: u32, my: u32) {
        let zone_index = self.map_to_zone_index(mx, my);

        // TODO: beware wrapping errors?
        self.zones[zone_index].num_blocks -= 1;
    }

    pub fn add_player_at_map_xy(&mut self, mx: u32, my: u32) {
        let zone_index = self.map_to_zone_index(mx, my);
        self.zones[zone_index].num_players += 1;
    }

    pub fn clear_players(&mut self) {
        for zone in &mut self.zones {
            zone.num_players = 0;
        }
    }

    pub fn get_zone_at_index(&self, index: usize) -> &WorldZone {
        &self.zones[index]
    }

    pub fn get_zone_at_index_mut(&mut self, index: usize) -> &mut WorldZone {
        &mut self.zones[index]
    }

    pub fn get_zone_at(&self, zone_x: u32, zone_y: u32) -> &WorldZone {
        let index = self.get_zone_index(zone_x, zone_y);
        self.get_zone_at_index(index)
    }

    pub fn get_zone_at_mut(&mut self, zone_x: u32, zone_y: u32) -> &mut WorldZone {
        let index = self.get_zone_index(zone_x, zone_y);
        self.get_zone_at_index_mut(index)
    }
}
