use rand::prelude::*;

pub trait RandEnum: Clone + Sized {
    fn get_enum_values() -> Vec<Self>;

    fn random() -> Self {
        let v = Self::get_enum_values();
        let index = rand::rng().random_range(0..v.len());
        v[index].clone()
    }
}

pub trait RandEnumFrom<T: Copy>: From<T> + Sized {
    fn get_enum_values() -> Vec<T>;

    fn random() -> Self {
        let v = Self::get_enum_values();
        let index = rand::rng().random_range(0..v.len());
        Self::from(v[index])
    }
}
