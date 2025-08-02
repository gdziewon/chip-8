use minifb::{Key, KeyRepeat, Scale, Window, WindowOptions};
use crate::errors::Chip8Error;

pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;
pub const DISPLAY_SCALE: Scale = Scale::X16;
const WINDOW_NAME: &str = "Chip8 Emulator";

pub struct Display {
    grid: [[bool; DISPLAY_HEIGHT]; DISPLAY_WIDTH],
    window: Option<Window>,
    buffer: Vec<u32>,
    colors: Colors,
    scale: Scale
}

impl Display {
    pub fn new() -> Self {
        let grid = [[false; DISPLAY_HEIGHT]; DISPLAY_WIDTH];
        let buffer: Vec<u32> = vec![0; DISPLAY_WIDTH * DISPLAY_HEIGHT];
        let colors = Colors {
            filled: Color::from_u8(0xFF, 0xFF, 0xFF),
            empty: Color::from_u8(0, 0, 0)
        };

        Display { grid, buffer, window: None, colors, scale: DISPLAY_SCALE }
    }

    pub(super) fn init(&mut self) -> Result<(), Chip8Error> {
        let window = Window::new(
            WINDOW_NAME,
            DISPLAY_WIDTH,
            DISPLAY_HEIGHT,
            WindowOptions {
                resize: true,
                scale: self.scale,
                scale_mode: minifb::ScaleMode::AspectRatioStretch,
                ..WindowOptions::default()
            },
        )
        .map_err(Chip8Error::WindowCreationError)?;

        self.window = Some(window);
        Ok(())
    }

    // Get the key pressed by the user
    pub fn get_key_press(&self, keyboard: &super::Keys) -> Option<u8> {
        self.window.as_ref().unwrap().get_keys_pressed(KeyRepeat::No)
        .iter()
        .find_map(|&k| keyboard.get_chip8_key(&k))
        .copied()
    }

    // Check if a key is pressed
    pub(super) fn is_key_down(&self, key: Key) -> bool {
        self.window.as_ref().unwrap().is_key_down(key)
    }

    // Check if the window is open
    pub(super) fn is_open(&self) -> bool {
        match self.window.as_ref() {
            Some(window) => window.is_open(),
            None => false,
        }
    }

    // Set color palette for the display
    pub(super) fn set_colors(&mut self, filled: Color, empty: Color) {
        self.colors.filled = filled;
        self.colors.empty = empty;
    }

    // Update the display
    pub(super) fn update(&mut self) -> Result<(), Chip8Error>{
        // Draw a grid
        self.update_buffer();

        // Update the window with buffer
        self.window.as_mut().unwrap()
            .update_with_buffer(&self.buffer, DISPLAY_WIDTH, DISPLAY_HEIGHT)
            .map_err(Chip8Error::WindowUpdateError)

    }

    // Clear the display
    pub(super) fn clear(&mut self) {
        self.grid = [[false; DISPLAY_HEIGHT]; DISPLAY_WIDTH];
        self.update_buffer();
    }

    pub fn close(&mut self) {
        self.window = None;
    }

    pub fn get_grid(&self) -> &[[bool; DISPLAY_HEIGHT]; DISPLAY_WIDTH] {
        &self.grid
    }

    // Draw a sprite on the display
    pub(super) fn draw(&mut self, horizontal_pos: usize, vertical_pos: usize, sprite: impl Iterator<Item = u8>) -> bool {
        let mut collision = false;
        for (j, byte) in sprite.enumerate() {
            for i in 0..8 {
                let xi = (horizontal_pos + i) % DISPLAY_WIDTH;
                let yj = (vertical_pos + j) % DISPLAY_HEIGHT;
                let old = self.grid[xi][yj];
                let new = (byte & (0x80 >> i)) != 0;
                self.grid[xi][yj] ^= new;
                collision |= old && !self.grid[xi][yj];
            }
        }
        collision
    }

    pub fn set_scale(&mut self, scale: Scale) {
        self.scale = scale;
    }

    pub fn get_scale(&self) -> Scale {
        self.scale
    }

    // Update buffer with grid
    fn update_buffer(&mut self) {
        for i in 0..DISPLAY_WIDTH {
            for j in 0..DISPLAY_HEIGHT {
                let color = if self.grid[i][j] { &self.colors.filled } else { &self.colors.empty };
                self.buffer[i + j * DISPLAY_WIDTH] = color.value();
            }
        }
    }


}

struct Colors {
    filled: Color,
    empty: Color
}

pub struct Color {
    value: u32,
}

impl Color {
    pub fn from_u8(r: u8, g: u8, b: u8) -> Self {
        let value = ((r as u32) << 16) | ((g as u32) << 8) | b as u32;
        Self { value }
    }

    fn value(&self) -> u32 {
        self.value
    }
}