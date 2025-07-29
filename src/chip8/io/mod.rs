mod display;
mod keys;
mod audio;

use std::collections::HashMap;

use display::Display;
use keys::Keys;
use audio::Audio;
use minifb::{Key, Scale};

use crate::errors::Chip8Error;

pub struct IO {
    pub(crate) keyboard: Keys,
    pub(crate) audio: Audio,
    pub(crate) display: Display
}

impl IO {
    pub fn new() -> Self {
        let keyboard = Keys::get_default();
        let display = Display::new();
        let audio = Audio::new();
        IO { keyboard, audio, display }
    }

    pub fn update(&mut self, st: u8) {
        self.display.update();

        if st > 0 {
            self.audio.play();
        } else {
            self.audio.pause();
        }
    }

    pub fn keyboard_set_bindings(&mut self, bindings: HashMap<u8, Key>) {
        self.keyboard = Keys::from(bindings);
    }

    pub fn display_set_scale(&mut self, scale: Scale) {
        self.display.set_scale(scale);
    }

    pub fn display_set_colors(&mut self, filled: u32, empty: u32) {
        self.display.set_colors(filled, empty);
    }

    pub fn display_init(&mut self) -> Result<(), Chip8Error> {
        self.display.init()
    }

    pub fn display_clear(&mut self) {
        self.display.clear();
    }

    pub fn display_draw(&mut self, horizontal_pos: usize, vertical_pos: usize, sprite: impl Iterator<Item = u8>) -> bool {
        self.display.draw(horizontal_pos, vertical_pos, sprite)
    }

    pub fn display_is_open(&self) -> bool {
        self.display.is_open()
    }

    pub fn get_key_press(&self) -> Option<u8> {
        self.display.get_key_press(&self.keyboard)
    }

    pub fn is_key_down(&self, key: u8) -> bool {
        if let Some(key) = self.keyboard.get_by_value(key) {
            return self.display.is_key_down(*key)
        }
        return false;
    }

    pub fn display_update(&mut self) -> Result<(), crate::errors::Chip8Error> {
        self.display.update()
    }

    pub fn audio_play(&self) {
        self.audio.play();
    }

    pub fn audio_pause(&self) {
        self.audio.pause();
    }
}