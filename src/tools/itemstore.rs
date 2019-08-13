use std::collections::HashMap;
use std::iter;

pub trait HasId {
    fn set_id(&mut self, id: u32);
}

#[derive(Debug)]
pub struct ItemStore<T: HasId> {
    items: HashMap<u32, T>,
    next_id: u32,
}

impl<T: HasId> Default for ItemStore<T> {
    fn default() -> Self {
        ItemStore {
            items: HashMap::new(),
            next_id: 1, // NOTE: there is no id 0.
        }
    }
}

impl<T: HasId> ItemStore<T> {
    pub fn new() -> Self {
        ItemStore::default()
    }

    fn get_next_id(&mut self) -> u32 {
        let next = self.next_id;
        self.next_id += 1;
        next
    }

    pub fn add(&mut self, item: T) -> u32 {
        let id = self.get_next_id();
        let mut item = item;
        item.set_id(id);
        self.items.insert(id, item);
        id
    }

    pub fn destroy(&mut self, id: u32) {
        if self.items.contains_key(&id) {
            self.items.remove(&id);
        }
    }

    pub fn get(&self, id: u32) -> Option<&T> {
        self.items.get(&id)
    }

    pub fn replace(&mut self, id: u32, item: T) {
        let mut item = item;
        item.set_id(id);
        self.items.insert(id, item);
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.values()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.items.values_mut()
    }

    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&u32, &mut T) -> bool,
    {
        self.items.retain(f);
    }
}
