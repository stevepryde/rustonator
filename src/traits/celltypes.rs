use rand::Rng;

#[derive(Copy, Clone, Debug)]
pub enum CellType {
    Empty = 0,
    Wall = 1,
    Mystery = 2,
    ItemBomb = 3,
    ItemRange = 4,
    ItemRandom = 5,
    MobSpawner = 6,
    Bomb = 100,
}

impl CellType {
    pub fn from(value: u8) -> Self {
        match value {
            0 => CellType::Empty,
            1 => CellType::Wall,
            2 => CellType::Mystery,
            3 => CellType::ItemBomb,
            4 => CellType::ItemRange,
            5 => CellType::ItemRandom,
            6 => CellType::MobSpawner,
            7 => CellType::Bomb, // To allow random to work.
            100 => CellType::Bomb,
            _ => panic!("Invalid cell type: {}", value),
        }
    }

    // TODO: do I need this?
    pub fn random() -> Self {
        CellType::from(rand::thread_rng().gen_range(0, 8))
    }
}

pub trait CanPass {
    fn can_pass(&self, _cell_type: CellType) -> bool {
        false
    }
}
