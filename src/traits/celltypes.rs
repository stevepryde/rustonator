use crate::{
    engine::{position::MapPosition, world::World},
    traits::randenum::RandEnumFrom,
};

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

impl From<u8> for CellType {
    fn from(value: u8) -> Self {
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
}

// TODO: do I need this?
impl RandEnumFrom<u8> for CellType {
    fn get_enum_values() -> Vec<u8> {
        let mut v: Vec<u8> = (0..8).collect();
        v.push(100);
        v
    }
}

pub trait CanPass {
    fn can_pass(&self, position: MapPosition, world: &World) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random() {
        let r = CellType::random();
        println!("{:?}", r);
    }
}
