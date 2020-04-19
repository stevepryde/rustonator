use crate::{
    engine::{
        bomb::{Bomb, BombId},
        config::GameConfig,
        explosion::Explosion,
        position::{MapPosition, PositionOffset, SizeInPixels, SizeInTiles},
        types::{BombList, ExplosionList, PlayerList},
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
    traits::celltypes::{CanPass, CellType},
    utils::misc::Timestamp,
};
use log::*;
use rand::Rng;
use serde::Serialize;
use std::collections::{HashSet, VecDeque};

#[derive(Debug, Serialize)]
pub struct WorldSize {
    map_size: SizeInTiles,
    tile_size: SizeInPixels,
    chunk_size: SizeInTiles,
}

impl WorldSize {
    pub fn new(width: i32, height: i32, config: &GameConfig) -> Self {
        let tile_width = 32;
        let tile_height = 32;

        // Width and height must be odd.
        let map_width = if width % 2 == 0 { width + 1 } else { width };
        let map_height = if height % 2 == 0 { height + 1 } else { height };

        let chunk_width = (config.screen_width() as f64 / tile_width as f64) as i32 + 10;
        let chunk_height = (config.screen_height() as f64 / tile_height as f64) as i32 + 10;

        WorldSize {
            map_size: SizeInTiles::new(map_width, map_height),
            tile_size: SizeInPixels::new(tile_width, tile_height),
            chunk_size: SizeInTiles::new(chunk_width, chunk_height),
        }
    }

    pub fn map_size(&self) -> &SizeInTiles {
        &self.map_size
    }

    pub fn tile_size(&self) -> &SizeInPixels {
        &self.tile_size
    }

    pub fn chunk_size(&self) -> &SizeInTiles {
        &self.chunk_size
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
    pub fn new(width: i32, height: i32, config: &GameConfig) -> Self {
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

            if y % 2 == 0 {
                for x in 1..((world.sizes.map_size.width as f64 / 2.0) as i32) {
                    world.set_cell(MapPosition::new(x * 2, y), CellType::Wall);
                }
            }
        }

        world
    }

    pub fn sizes(&self) -> &WorldSize {
        &self.sizes
    }

    pub fn data(&self) -> &WorldData {
        &self.data
    }

    pub fn zones(&self) -> &WorldZoneData {
        &self.zones
    }

    pub fn zones_mut(&mut self) -> &mut WorldZoneData {
        &mut self.zones
    }

    fn get_index(&self, pos: MapPosition) -> usize {
        ((pos.y * self.sizes.map_size.width) + pos.x) as usize
    }

    pub fn get_cell(&self, pos: MapPosition) -> Option<CellType> {
        self.data.get_at(pos).map(CellType::from)
    }

    pub fn set_cell(&mut self, pos: MapPosition, value: CellType) {
        if let Some(CellType::Mystery) = self.get_cell(pos) {
            self.zones.del_block_at_map_xy(pos);
        }
        if let CellType::Mystery = value {
            self.zones.add_block_at_map_xy(pos);
        }
        self.data.set_at(pos, value as u8);
    }

    /// This is here for debugging only.
    fn _validate_pos(&self, p: MapPosition, text: &str) {
        if p.x <= 0
            || p.x >= self.sizes().map_size().width - 1
            || p.y <= 0
            || p.y >= self.sizes().map_size().height - 1
        {
            error!("Got invalid pos: {:?} ({})", p, text);
        }
    }

    pub fn find_nearest_blank(&self, pos: MapPosition) -> MapPosition {
        // clamp pos.

        if let Some(CellType::Empty) = self.get_cell(pos) {
            return pos;
        }

        for radius in 1..20 {
            // Top.
            if pos.y - radius > 0 {
                for offset_x in -radius..=radius {
                    let xx = pos.x + offset_x;
                    if xx <= 0 || xx >= self.sizes.map_size.width - 1 {
                        continue;
                    }

                    let p = pos + PositionOffset::new(offset_x, -radius);
                    if let Some(CellType::Empty) = self.get_cell(p) {
                        return p;
                    }
                }
            }

            // Bottom.
            if pos.y + radius < self.sizes.map_size.height - 1 {
                for offset_x in -radius..=radius {
                    let xx = pos.x + offset_x;
                    if xx <= 0 || xx >= self.sizes.map_size.width - 1 {
                        continue;
                    }

                    let p = pos + PositionOffset::new(offset_x, radius);
                    if let Some(CellType::Empty) = self.get_cell(p) {
                        return p;
                    }
                }
            }

            // Left.
            if pos.x - radius > 0 {
                for offset_y in -radius..=radius {
                    let yy = pos.y + offset_y;
                    if yy <= 0 || yy >= self.sizes.map_size.height - 1 {
                        continue;
                    }

                    let p = pos + PositionOffset::new(-radius, offset_y);
                    if let Some(CellType::Empty) = self.get_cell(p) {
                        return p;
                    }
                }
            }

            // Right.
            if pos.x + radius < self.sizes.map_size.width - 1 {
                for offset_y in -radius..=radius {
                    let yy = pos.y + offset_y;
                    if yy <= 0 || yy >= self.sizes.map_size.height - 1 {
                        continue;
                    }

                    let p = pos + PositionOffset::new(-radius, offset_y);
                    if let Some(CellType::Empty) = self.get_cell(p) {
                        return p;
                    }
                }
            }
        }

        MapPosition::new(1, 1)
    }

    pub fn get_internal_cell(&self, pos: MapPosition) -> Option<&InternalCellData> {
        self.data_internal.get_at(pos)
    }

    pub fn clear_internal_cell(&mut self, pos: MapPosition) {
        self.data_internal.set_at(pos, InternalCellData::Empty);
    }

    pub fn set_mob_data(&mut self, pos: MapPosition, timestamp: Timestamp) {
        match self.data_mob.get_at(pos) {
            Some(ts) => {
                // Only overwrite the bomb timestamp if it's sooner
                // i.e. if this one was deployed before the existing one
                if timestamp < ts {
                    self.data_mob.set_at(pos, Some(timestamp));
                }
            }
            None => self.data_mob.set_at(pos, Some(timestamp)),
        }
    }

    pub fn get_mob_data(&self, pos: MapPosition) -> Option<Timestamp> {
        self.data_mob.get_at(pos)
    }

    pub fn get_mob_data_update(&mut self, pos: MapPosition) -> Option<Timestamp> {
        self.data_mob.get_at_fix(pos)
    }

    pub fn clear_mob_data(&mut self, pos: MapPosition) {
        self.data_mob.set_at(pos, None);
    }

    pub fn get_spawn_point(&self) -> MapPosition {
        for _ in 0..1000 {
            let tx = rand::thread_rng().gen_range(0, self.sizes.map_size.width);
            let ty = rand::thread_rng().gen_range(0, self.sizes.map_size.height);
            let pos = self.find_nearest_blank(MapPosition::new(tx, ty));

            let mut count = 0;
            if let Some(CellType::Empty) = self.get_cell(MapPosition::new(pos.x - 1, pos.y)) {
                count += 1;
            }
            if let Some(CellType::Empty) = self.get_cell(MapPosition::new(pos.x + 1, pos.y)) {
                count += 1;
            }
            if let Some(CellType::Empty) = self.get_cell(MapPosition::new(pos.x, pos.y - 1)) {
                count += 1;
            }
            if let Some(CellType::Empty) = self.get_cell(MapPosition::new(pos.x, pos.y + 1)) {
                count += 1;
            }

            if count >= 2 {
                return pos;
            }
        }

        MapPosition::new(1, 1)
    }

    pub fn get_chunk_data(&self, position: MapPosition) -> WorldChunk {
        let halfwidth = self.sizes.chunk_size.width / 2;
        let halfheight = self.sizes.chunk_size.height / 2;
        let maxx = self.sizes.map_size.width - self.sizes.chunk_size.width;
        let maxy = self.sizes.map_size.height - self.sizes.chunk_size.height;
        let mut topleft = position + PositionOffset::new(-halfwidth, -halfheight);

        // Clamp rect to map bounds.
        if topleft.x < 0 {
            topleft.x = 0;
        } else if topleft.x > maxx {
            topleft.x = maxx;
        }
        if topleft.y < 0 {
            topleft.y = 0;
        } else if topleft.y > maxy {
            topleft.y = maxy;
        }

        let mut chunk = WorldChunk::new(
            topleft.x,
            topleft.y,
            self.sizes.chunk_size.width,
            self.sizes.chunk_size.height,
        );

        let mut index_read = self.get_index(MapPosition::new(topleft.x, topleft.y));
        let mut index_write = 0 as usize;
        for _ in 0..self.sizes.chunk_size.height {
            chunk.set_slice(
                index_write,
                self.data
                    .get_slice(index_read, self.sizes.chunk_size.width as usize),
            );

            index_read += self.sizes.map_size.width as usize;
            index_write += self.sizes.chunk_size.width as usize;
        }

        chunk
    }

    pub fn is_nearby_map_entity(
        &self,
        pos: MapPosition,
        entities: &[MapPosition],
        range: i32,
    ) -> bool
    {
        entities.iter().any(|e| e.is_within_range(pos, range))
    }

    pub fn populate_blocks(&mut self, map_positions: &[MapPosition]) {
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

            if !self.is_nearby_map_entity(blank, map_positions, 4) {
                new_blocks.insert(blank);
                break;
            }
        }

        // Now add the blocks.
        for block in new_blocks {
            self.set_cell(block, CellType::Mystery);
        }
    }

    pub fn populate_initial(&mut self, map_positions: &[MapPosition]) {
        let mut new_blocks = HashSet::new();
        for zone in self.zones.zone_iter() {
            for _ in 0..zone.quota() {
                let bx = rand::thread_rng().gen_range(0, zone.size().width) + zone.position().x;
                let by = rand::thread_rng().gen_range(0, zone.size().height) + zone.position().y;
                let blank = self.find_nearest_blank(MapPosition::new(bx, by));

                // Avoid top left corner - it's the safe space for spawning players if no blank
                // spaces were found.
                if blank.x == 1 && blank.y == 1 {
                    continue;
                }

                if new_blocks.contains(&blank) {
                    continue;
                }

                if !self.is_nearby_map_entity(blank, map_positions, 4) {
                    new_blocks.insert(blank);
                }
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
        let stepx = self.sizes.map_size.width as f64 / numx as f64;
        let stepy = self.sizes.map_size.height as f64 / numy as f64;
        let half_stepx = stepx / 2.0;
        let half_stepy = stepy / 2.0;
        let mut mob_spawners = Vec::new();

        for py in 0..numx {
            for px in 0..numy {
                let mx = ((stepx * px as f64) + half_stepx) as i32;
                let my = ((stepy * py as f64) + half_stepy) as i32;

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

    pub fn add_bomb(&mut self, bomb: Bomb, bombs: &mut BombList) {
        let pos = bomb.position();
        let id = bombs.add(bomb);
        self.set_cell(pos, CellType::Bomb);
        self.update_bomb_path(id, &bombs);
        self.data_internal.set_at(pos, InternalCellData::Bomb(id));
    }

    pub fn add_explosion(&mut self, explosion: Explosion, explosions: &mut ExplosionList) {
        let pos = explosion.position();
        let id = explosions.add(explosion);
        self.data_internal
            .set_at(pos, InternalCellData::Explosion(id));
    }

    pub fn add_visual_only_explosion(&mut self, pos: MapPosition, explosions: &mut ExplosionList) {
        explosions.add(Explosion::from(pos));
    }

    pub fn clear_explosion_cell(&mut self, explosion: &Explosion) {
        let pos = explosion.position();
        self.clear_internal_cell(pos);

        // Also let mobs know it's "safe" here now
        if let Some(ts) = self.data_mob.get_at(pos) {
            if explosion.timestamp() >= ts {
                self.clear_mob_data(pos);
            }
        }
    }

    /// This will walk the entire bomb path, collecting the soonest explosion
    /// time along the way. Once found, it will update the mob data
    /// timestamp to that earliest time for all of the positions it finds.
    /// This is because an explosion of any one of these bombs will
    /// result in all of them going boom, thus mobs need to know _that_ time not
    /// just the timestamp for the nearest bomb.
    pub fn update_bomb_path(&mut self, bid: BombId, bombs: &BombList) {
        let mut bombs_to_follow: VecDeque<BombId> = VecDeque::new();
        bombs_to_follow.push_back(bid);
        let mut seen: HashSet<MapPosition> = HashSet::new();
        let mut earliest_ts = match bombs.get(bid) {
            Some(b) => {
                seen.insert(b.position());
                b.timestamp()
            }
            None => {
                return;
            }
        };
        let mut path_cells = Vec::new();

        while let Some(bomb_id) = bombs_to_follow.pop_front() {
            if let Some(b) = bombs.get(bomb_id) {
                if b.timestamp() < earliest_ts {
                    earliest_ts = b.timestamp();
                }

                for offset in vec![
                    PositionOffset::up(1),
                    PositionOffset::down(1),
                    PositionOffset::left(1),
                    PositionOffset::right(1),
                ]
                .into_iter()
                {
                    for dist in 1..=*b.range() {
                        let pos = b.position() + (offset * dist as i32);
                        if seen.contains(&pos) {
                            continue;
                        }
                        seen.insert(pos);

                        match self.get_cell(pos) {
                            Some(CellType::Bomb) => {
                                if let Some(InternalCellData::Bomb(bomb_id)) =
                                    self.data_internal.get_at(pos)
                                {
                                    bombs_to_follow.push_back(*bomb_id);
                                }
                            }
                            // Explosions can pass through the following.
                            Some(CellType::Empty)
                            | Some(CellType::ItemBomb)
                            | Some(CellType::ItemRange)
                            | Some(CellType::ItemRandom)
                            | Some(CellType::MobSpawner) => {
                                path_cells.push(pos);
                            }
                            // The following will block an explosion, so stop.
                            Some(CellType::Wall) | Some(CellType::Mystery) | None => break,
                        }
                    }
                }
            }
        }

        // Now set the earliest timestamp at all locations!
        for pos in path_cells {
            self.set_mob_data(pos, earliest_ts);
        }
    }

    pub fn explode_bomb(
        &mut self,
        bomb_id: BombId,
        bombs: &mut BombList,
        explosions: &mut ExplosionList,
        players: &mut PlayerList,
    )
    {
        let mut bombs_to_explode: VecDeque<BombId> = VecDeque::new();
        bombs_to_explode.push_back(bomb_id);
        while let Some(bomb_id) = bombs_to_explode.pop_front() {
            if let Some(b) = bombs.get_mut(bomb_id) {
                if let Some(CellType::Bomb) = self.get_cell(b.position()) {
                    self.set_cell(b.position(), CellType::Empty);
                    self.clear_internal_cell(b.position());
                }

                let bombs_cascade = self.explode_bomb_path(b, explosions);
                // Update player bomb count.
                if let Some(p) = players.get_mut(&b.pid()) {
                    p.bomb_exploded();
                }

                b.terminate();
                bombs_to_explode.extend(bombs_cascade);
            }
        }
    }

    pub fn explode_bomb_path(
        &mut self,
        bomb: &Bomb,
        explosions: &mut ExplosionList,
    ) -> Vec<BombId>
    {
        self.add_explosion(Explosion::from((bomb.clone(), bomb.position())), explosions);

        let mut bombs_cascade = Vec::new();

        for offset in vec![
            PositionOffset::up(1),
            PositionOffset::down(1),
            PositionOffset::left(1),
            PositionOffset::right(1),
        ]
        .into_iter()
        {
            for dist in 1..=*bomb.range() {
                let pos = bomb.position() + (offset * dist as i32);
                match self.get_cell(pos) {
                    // Explosions will in turn explode other bombs.
                    Some(CellType::Bomb) => {
                        if let Some(InternalCellData::Bomb(bomb_id)) =
                            self.data_internal.get_at(pos)
                        {
                            bombs_cascade.push(*bomb_id);
                        } else {
                            // Can't find bomb? Might as well assume the cell is empty.
                            self.add_explosion(Explosion::from((bomb.clone(), pos)), explosions);
                        }
                        break;
                    }
                    // Explosions can pass through the following.
                    Some(CellType::ItemBomb)
                    | Some(CellType::ItemRange)
                    | Some(CellType::ItemRandom) => {
                        self.add_explosion(Explosion::from((bomb.clone(), pos)), explosions);
                        self.set_cell(pos, CellType::Empty);
                    }
                    Some(CellType::Empty) | Some(CellType::MobSpawner) => {
                        self.add_explosion(Explosion::from((bomb.clone(), pos)), explosions);
                    }

                    // The following will block an explosion, so stop.
                    Some(CellType::Mystery) => {
                        // This will become a powerup item.
                        let r: f64 = rand::thread_rng().gen();
                        let item = if r > 0.9 {
                            // 10% chance.
                            CellType::ItemBomb
                        } else if r > 0.8 {
                            // 10% chance.
                            CellType::ItemRange
                        } else if r > 0.5 {
                            // 30% chance.
                            // Mystery item. Contents are determined at random when player picks
                            // it up.
                            CellType::ItemRandom
                        } else {
                            CellType::Empty
                        };
                        self.add_explosion(Explosion::from((bomb.clone(), pos)), explosions);
                        self.set_cell(pos, item);
                        break;
                    }
                    Some(CellType::Wall) | None => break,
                }
            }
        }
        bombs_cascade
    }
}

pub struct PathFindData {
    position: MapPosition,
    travelled: u32,
    initial_offset: Option<PositionOffset>,
}

impl PathFindData {
    pub fn new(position: MapPosition) -> Self {
        Self {
            position,
            travelled: 0,
            initial_offset: None,
        }
    }

    pub fn new_from(position: MapPosition, prev: &PathFindData) -> Self {
        Self {
            position,
            travelled: prev.travelled + 1,
            initial_offset: prev
                .initial_offset
                .or_else(|| Some(position - prev.position)),
        }
    }
}

impl World {
    pub fn get_possible_moves<T>(
        &self,
        agent: &T,
        pf: &PathFindData,
        seen: &HashSet<MapPosition>,
    ) -> Vec<PathFindData>
    where
        T: CanPass,
    {
        let pos = pf.position;
        let mut possible_moves = Vec::new();
        for m in &[pos.up(1), pos.right(1), pos.down(1), pos.left(1)] {
            if seen.contains(&m) {
                continue;
            }

            if agent.can_pass(*m, self) {
                possible_moves.push(PathFindData::new_from(*m, pf));
            }
        }
        possible_moves
    }

    pub fn path_find<T>(
        &self,
        agent: &T,
        pos_from: MapPosition,
        pos_to: MapPosition,
        range: u32,
    ) -> Option<PositionOffset>
    where
        T: CanPass,
    {
        if pos_to == pos_from {
            return None;
        }

        let mut open_list: Vec<PathFindData> = Vec::with_capacity(4);
        open_list.push(PathFindData::new(pos_from));

        let mut seen: HashSet<MapPosition> = HashSet::new();
        seen.insert(pos_from);

        while !open_list.is_empty() {
            open_list.sort_by_cached_key(|a| a.travelled + a.position.distance_to(pos_to));
            let mut processed = 0;
            for element in &open_list {
                if element.position == pos_to {
                    return element.initial_offset;
                }
                seen.insert(element.position);
                processed += 1;
                if element.travelled < range {
                    let mut new_moves = self.get_possible_moves(agent, element, &seen);
                    if !new_moves.is_empty() {
                        open_list.append(&mut new_moves);
                        // Since we've added new elements, sort the list before going further.
                        break;
                    }
                }
            }

            // Remove processed items.
            open_list = open_list.split_off(processed);
        }

        None
    }

    pub fn path_find_nearest_safe_space<T>(
        &self,
        agent: &T,
        pos: MapPosition,
        range: u32,
    ) -> MapPosition
    where
        T: CanPass,
    {
        if self.get_mob_data(pos).is_none() {
            return pos;
        }

        let mut open_list: Vec<PathFindData> = Vec::with_capacity(4);
        open_list.push(PathFindData::new(pos));

        let mut seen: HashSet<MapPosition> = HashSet::new();
        seen.insert(pos);

        // Um, a timestamp of now definitely isn't safe! let's hope we find a better
        // one.
        let mut safest_timestamp = Timestamp::default();
        let mut safest_pos = pos;

        while !open_list.is_empty() {
            open_list.sort_by_cached_key(|a| a.travelled);

            let mut processed = 0;
            for element in &open_list {
                match self.get_mob_data(element.position) {
                    None => {
                        return element.position;
                    }
                    Some(ts) => {
                        if ts > safest_timestamp {
                            safest_timestamp = ts;
                            safest_pos = element.position;
                        }
                    }
                }

                seen.insert(element.position);

                processed += 1;
                if element.travelled < range {
                    let mut new_moves = self.get_possible_moves(agent, element, &seen);
                    if !new_moves.is_empty() {
                        open_list.append(&mut new_moves);
                        // Since we've added new elements, sort the list before going further.
                        break;
                    }
                }
            }

            // Remove processed items.
            open_list = open_list.split_off(processed);
        }

        safest_pos
    }
}
