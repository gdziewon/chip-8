use std::collections::HashMap;
use minifb::Key;

pub(super) struct Keys {
    left: HashMap<u8, Key>,
    right: HashMap<Key, u8>,
}

impl Keys {
    pub fn new() -> Self {
        let left = HashMap::new();
        let right = HashMap::new();
        Keys { left, right }
    }

    pub fn from(bindings: HashMap<u8, Key>) -> Self {
        let (left, right) = Self::create_bindings(bindings);
        Keys { left, right }
    }

    pub fn set_bindings(&mut self, bindings: HashMap<u8, Key>) {
        let (left, right) = Self::create_bindings(bindings);
        self.left = left;
        self.right = right;
    }

    pub fn get_bindings(&self) -> HashMap<u8, Key> {
        self.left.clone()
    }

    pub fn get_by_key(&self, key: &Key) -> Option<&u8> {
        self.right.get(key)
    }
    
    pub fn get_by_value(&self, value: u8) -> Option<&Key> {
        self.left.get(&value)
    }

    pub fn insert(&mut self, key: u8, value: Key) {
        if let Some(old_value) = self.left.insert(key, value) {
            self.right.remove(&old_value);
        }
        self.right.insert(value, key);
    }

    fn create_bindings(bindings: HashMap<u8, Key>) -> (HashMap<u8, Key>, HashMap<Key, u8>) {
        let mut left = HashMap::new();
        let mut right = HashMap::new();
        bindings.iter().for_each(|(k, v)| {
            left.insert(*k, *v);
            right.insert(*v, *k);
        });
        (left, right)
    }

    pub fn get_default() -> Self {
        let mut bindings = HashMap::new();
        bindings.insert(0x1, Key::Key1);
        bindings.insert(0x2, Key::Key2);
        bindings.insert(0x3, Key::Key3);
        bindings.insert(0xC, Key::Key4);
        bindings.insert(0x4, Key::Q);
        bindings.insert(0x5, Key::W);
        bindings.insert(0x6, Key::E);
        bindings.insert(0xD, Key::R);
        bindings.insert(0x7, Key::A);
        bindings.insert(0x8, Key::S);
        bindings.insert(0x9, Key::D);
        bindings.insert(0xE, Key::F);
        bindings.insert(0xA, Key::Z);
        bindings.insert(0x0, Key::X);
        bindings.insert(0xB, Key::C);
        bindings.insert(0xF, Key::V);
        Keys::from(bindings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use minifb::Key;

    #[test]
    fn test_get_by_key() {
        let mut keys = Keys::new();
        keys.insert(0x1, Key::Key1);
        assert_eq!(keys.get_by_key(&Key::Key1), Some(&0x1));
        assert_eq!(keys.get_by_key(&Key::Key2), None);
    }

    #[test]
    fn test_get_by_value() {
        let mut keys = Keys::new();
        keys.insert(0x1, Key::Key1);
        assert_eq!(keys.get_by_value(0x1), Some(&Key::Key1));
        assert_eq!(keys.get_by_value(0x2), None);
    }

    #[test]
    fn test_insert() {
        let mut keys = Keys::new();
        keys.insert(0x1, Key::Key1);
        assert_eq!(keys.get_by_key(&Key::Key1), Some(&0x1));
        assert_eq!(keys.get_by_value(0x1), Some(&Key::Key1));
        keys.insert(0x1, Key::Key2);
        assert_eq!(keys.get_by_key(&Key::Key1), None);
        assert_eq!(keys.get_by_key(&Key::Key2), Some(&0x1));
        assert_eq!(keys.get_by_value(0x1), Some(&Key::Key2));
    }

    #[test]
    fn test_from() {
        let mut bindings = HashMap::new();
        bindings.insert(0x1, Key::Key1);
        bindings.insert(0x2, Key::Key2);
        let keys = Keys::from(bindings);
        assert_eq!(keys.get_by_key(&Key::Key1), Some(&0x1));
        assert_eq!(keys.get_by_key(&Key::Key2), Some(&0x2));
        assert_eq!(keys.get_by_value(0x1), Some(&Key::Key1));
        assert_eq!(keys.get_by_value(0x2), Some(&Key::Key2));
    }

    #[test]
    fn test_set_bindings() {
        let mut keys = Keys::new();
        let mut bindings = HashMap::new();
        bindings.insert(0x1, Key::Key1);
        bindings.insert(0x2, Key::Key2);
        keys.set_bindings(bindings);
        assert_eq!(keys.get_by_key(&Key::Key1), Some(&0x1));
        assert_eq!(keys.get_by_key(&Key::Key2), Some(&0x2));
        assert_eq!(keys.get_by_value(0x1), Some(&Key::Key1));
        assert_eq!(keys.get_by_value(0x2), Some(&Key::Key2));
    }
}