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

    pub fn get_slice(&self, index: usize, length: usize) -> &[u8] {
        &self.0[index..(index + length)]
    }

    pub fn set_slice(&mut self, index: usize, slice: &[u8]) {
        let end = index + slice.len();
        self.0.as_mut_slice()[index..end].copy_from_slice(slice);
    }
}

#[derive(Debug, Clone)]
pub struct WorldChunk {
    tx: u32,
    ty: u32,
    width: u32,
    height: u32,
    data: WorldData,
}

impl WorldChunk {
    pub fn new(tx: u32, ty: u32, width: u32, height: u32) -> Self {
        WorldChunk {
            tx,
            ty,
            width,
            height,
            data: WorldData::new(width, height),
        }
    }

    pub fn set_slice(&mut self, index: usize, slice: &[u8]) {
        self.data.set_slice(index, slice);
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
