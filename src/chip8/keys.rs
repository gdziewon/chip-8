use std::collections::HashMap;
use minifb::Key;

pub struct Keys {
    left: HashMap<u8, Key>,
    right: HashMap<Key, u8>,
}

impl Keys {
    pub fn new(bindings: HashMap<u8, Key>) -> Self {
        let mut left = HashMap::new();
        let mut right = HashMap::new();
        bindings.iter().for_each(|(k, v)| {
            left.insert(*k, *v);
            right.insert(*v, *k);
        });
        Keys { left, right }
    }

    pub fn get_by_key(&self, key: &Key) -> Option<&u8> {
        self.right.get(key)
    }
    
    pub fn get_by_value(&self, value: u8) -> Option<&Key> {
        self.left.get(&value)
    }
}