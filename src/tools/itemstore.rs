use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub trait HasId<I: From<u64>> {
    fn set_id(&mut self, id: I);
}

#[derive(Debug)]
pub struct ItemStore<I: From<u64> + Debug + Hash + Eq, T: HasId<I>> {
    items: HashMap<I, T>,
    next_id: u64,
}

impl<I: From<u64> + Debug + Hash + Eq, T: HasId<I>> Default for ItemStore<I, T> {
    fn default() -> Self {
        ItemStore {
            items: HashMap::new(),
            next_id: 1, // NOTE: there is no id 0.
        }
    }
}

impl<I: Clone + From<u64> + Debug + Hash + Eq, T: HasId<I>> ItemStore<I, T> {
    pub fn new() -> Self {
        ItemStore::default()
    }

    fn get_next_id(&mut self) -> I {
        let next = self.next_id;
        self.next_id += 1;
        next.into()
    }

    pub fn add(&mut self, item: T) -> I {
        let id = self.get_next_id();
        let mut item = item;
        item.set_id(id.clone());
        self.items.insert(id.clone(), item);
        id
    }

    pub fn destroy(&mut self, id: I) {
        if self.items.contains_key(&id) {
            self.items.remove(&id);
        }
    }

    pub fn get(&self, id: I) -> Option<&T> {
        self.items.get(&id)
    }

    pub fn get_mut(&mut self, id: I) -> Option<&mut T> {
        self.items.get_mut(&id)
    }

    pub fn replace(&mut self, id: I, item: T) {
        let mut item = item;
        item.set_id(id.clone());
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
        F: FnMut(&I, &mut T) -> bool,
    {
        self.items.retain(f);
    }
}
