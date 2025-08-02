use std::collections::HashMap;
use minifb::Key;

const DEFAULT_BINDINGS: [(u8, Key); 16] = [
    (0x1, Key::Key1),
    (0x2, Key::Key2),
    (0x3, Key::Key3),
    (0xC, Key::Key4),
    (0x4, Key::Q),
    (0x5, Key::W),
    (0x6, Key::E),
    (0xD, Key::R),
    (0x7, Key::A),
    (0x8, Key::S),
    (0x9, Key::D),
    (0xE, Key::F),
    (0xA, Key::Z),
    (0x0, Key::X),
    (0xB, Key::C),
    (0xF, Key::V),
];

pub(super) struct Keys {
    chip8_to_minifb: HashMap<u8, Key>,
    minifb_to_chip8: HashMap<Key, u8>,
}

impl Default for Keys {
    fn default() -> Self {
        Self::from(HashMap::from(DEFAULT_BINDINGS))
    }
}

impl Keys {
    pub fn from(bindings: HashMap<u8, Key>) -> Self {
        let (chip8_to_minifb, minifb_to_chip8) = Self::create_bidirectional_map(&bindings);
        Keys { chip8_to_minifb, minifb_to_chip8 }
    }

    pub fn get_chip8_key(&self, key: &Key) -> Option<&u8> {
        self.minifb_to_chip8.get(key)
    }

    pub fn get_minifb_key(&self, value: u8) -> Option<&Key> {
        self.chip8_to_minifb.get(&value)
    }

    pub fn insert(&mut self, key: u8, value: Key) {
        if let Some(old_value) = self.chip8_to_minifb.insert(key, value) {
            self.minifb_to_chip8.remove(&old_value);
        }
        self.minifb_to_chip8.insert(value, key);
    }

    fn create_bidirectional_map(
        bindings: &HashMap<u8, Key>
    ) -> (HashMap<u8, Key>, HashMap<Key, u8>) {
        let mut chip8_to_minifb = HashMap::with_capacity(bindings.len());
        let mut minifb_to_chip8 = HashMap::with_capacity(bindings.len());

        for (&chip8_key, &minifb_key) in bindings {
            chip8_to_minifb.insert(chip8_key, minifb_key);
            minifb_to_chip8.insert(minifb_key, chip8_key);
        }

        (chip8_to_minifb, minifb_to_chip8)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use minifb::Key;

//     #[test]
//     fn test_get_by_key() {
//         let mut keys = Keys::new();
//         keys.insert(0x1, Key::Key1);
//         assert_eq!(keys.get_by_key(&Key::Key1), Some(&0x1));
//         assert_eq!(keys.get_by_key(&Key::Key2), None);
//     }

//     #[test]
//     fn test_get_by_value() {
//         let mut keys = Keys::new();
//         keys.insert(0x1, Key::Key1);
//         assert_eq!(keys.get_by_value(0x1), Some(&Key::Key1));
//         assert_eq!(keys.get_by_value(0x2), None);
//     }

//     #[test]
//     fn test_insert() {
//         let mut keys = Keys::new();
//         keys.insert(0x1, Key::Key1);
//         assert_eq!(keys.get_by_key(&Key::Key1), Some(&0x1));
//         assert_eq!(keys.get_by_value(0x1), Some(&Key::Key1));
//         keys.insert(0x1, Key::Key2);
//         assert_eq!(keys.get_by_key(&Key::Key1), None);
//         assert_eq!(keys.get_by_key(&Key::Key2), Some(&0x1));
//         assert_eq!(keys.get_by_value(0x1), Some(&Key::Key2));
//     }

//     #[test]
//     fn test_from() {
//         let mut bindings = HashMap::new();
//         bindings.insert(0x1, Key::Key1);
//         bindings.insert(0x2, Key::Key2);
//         let keys = Keys::from(bindings);
//         assert_eq!(keys.get_by_key(&Key::Key1), Some(&0x1));
//         assert_eq!(keys.get_by_key(&Key::Key2), Some(&0x2));
//         assert_eq!(keys.get_by_value(0x1), Some(&Key::Key1));
//         assert_eq!(keys.get_by_value(0x2), Some(&Key::Key2));
//     }

//     #[test]
//     fn test_set_bindings() {
//         let mut keys = Keys::new();
//         let mut bindings = HashMap::new();
//         bindings.insert(0x1, Key::Key1);
//         bindings.insert(0x2, Key::Key2);
//         keys.set_bindings(bindings);
//         assert_eq!(keys.get_by_key(&Key::Key1), Some(&0x1));
//         assert_eq!(keys.get_by_key(&Key::Key2), Some(&0x2));
//         assert_eq!(keys.get_by_value(0x1), Some(&Key::Key1));
//         assert_eq!(keys.get_by_value(0x2), Some(&Key::Key2));
//     }
// }