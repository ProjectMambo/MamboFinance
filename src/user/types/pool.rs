use crate::user::types::{HasLabel, Label};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

pub struct Pool<T> {
    items: RwLock<HashMap<Uuid, Arc<T>>>,
}

impl<T> Pool<T> {
    pub fn new() -> Self {
        Pool {
            items: RwLock::new(HashMap::new()),
        }
    }

    pub fn insert(&self, item: Arc<T>)
    where
        T: HasLabel,
    {
        let mut map = self.items.write().unwrap();
        map.insert(item.id(), item);
    }

    pub fn get(&self, label: &Label) -> Option<Arc<T>> {
        let map = self.items.read().unwrap();
        map.get(&label.id).cloned()
    }
}

impl<T> Default for Pool<T> {
    fn default() -> Self {
        Self::new()
    }
}
