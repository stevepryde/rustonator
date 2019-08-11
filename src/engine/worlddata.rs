#[derive(Debug, Clone)]
pub struct WorldData(Vec<u8>);

impl WorldData {
    pub fn new(width: u32, height: u32) -> Self {
        WorldData(vec![0; (width * height) as usize])
    }

    pub fn get_at_index(&self, index: usize) -> u8 {
        // TODO: do we need a range check here?
        self.0[index]
    }

    pub fn set_at_index(&mut self, index: usize, value: u8) {
        // TODO: do we need a range check here?
        self.0[index] = value;
    }
}

#[derive(Debug, Clone)]
pub enum InternalCellData {
    Empty,
    Bomb(u32),
    Explosion(u32),
}

#[derive(Debug, Clone)]
pub struct InternalWorldData(Vec<InternalCellData>);

impl InternalWorldData {
    pub fn new(width: u32, height: u32) -> Self {
        InternalWorldData(vec![InternalCellData::Empty; (width * height) as usize])
    }

    pub fn get_at_index(&self, index: usize) -> &InternalCellData {
        // TODO: do we need a range check here?
        &self.0[index]
    }

    pub fn set_at_index(&mut self, index: usize, value: InternalCellData) {
        // TODO: do we need a range check here?
        self.0[index] = value;
    }
}

#[derive(Debug, Clone)]
pub struct InternalMobData(Vec<i64>);

impl InternalMobData {
    pub fn new(width: u32, height: u32) -> Self {
        InternalMobData(vec![0; (width * height) as usize])
    }

    pub fn get_at_index(&self, index: usize) -> i64 {
        // TODO: do we need a range check here?
        self.0[index]
    }

    pub fn set_at_index(&mut self, index: usize, value: i64) {
        // TODO: do we need a range check here?
        self.0[index] = value;
    }
}

#[derive(Debug, Clone)]
pub struct MobSpawner {
    map_x: u32,
    map_y: u32,
}

impl MobSpawner {
    pub fn new(map_x: u32, map_y: u32) -> Self {
        MobSpawner { map_x, map_y }
    }
}
