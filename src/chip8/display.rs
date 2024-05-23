use minifb::{Window, WindowOptions, KeyRepeat, Key};
use super::{DISPLAY_HEIGHT, DISPLAY_WIDTH, DISPLAY_SCALE, WINDOW_NAME};
use super::errors::Chip8Error;

struct Colors {
    filled: u32,
    empty: u32
}

pub struct Display {
    grid: [[bool; DISPLAY_HEIGHT]; DISPLAY_WIDTH],
    window: Option<Window>,
    buffer: Vec<u32>,
    colors: Colors
}

impl Display {
    pub fn new() -> Self {
        let grid = [[false; DISPLAY_HEIGHT]; DISPLAY_WIDTH];
        let buffer: Vec<u32> = vec![0; DISPLAY_WIDTH * DISPLAY_HEIGHT];
        
        let colors = Colors {
            filled: 0xffffff,
            empty: 0x000000
        };
    
        Display { grid, buffer, window: None, colors }
    }

    pub(super) fn init(&mut self) -> Result<(), Chip8Error> {
        let window = Window::new(
            WINDOW_NAME,
            DISPLAY_WIDTH,
            DISPLAY_HEIGHT,
            WindowOptions {
                resize: true,
                scale: DISPLAY_SCALE,
                scale_mode: minifb::ScaleMode::AspectRatioStretch,
                ..WindowOptions::default()
            },
        )
        .map_err(Chip8Error::WindowCreationError)?;

        self.window = Some(window);
        Ok(())
    }

    pub(super) fn update_window(&mut self) {
        self.window.as_mut().unwrap().update();
    }

    pub(super) fn get_keys_pressed(&self) -> Vec<Key> {
        self.window.as_ref().unwrap().get_keys_pressed(KeyRepeat::No)
    }

    pub(super) fn is_key_down(&self, key: Key) -> bool {
        self.window.as_ref().unwrap().is_key_down(key)
    }

    pub(super) fn is_open(&self) -> bool {
        self.window.as_ref().unwrap().is_open()
    }

    pub(super) fn set_colors(&mut self, filled: u32, empty: u32) {
        self.colors.filled = filled;
        self.colors.empty = empty;
    }
    
    pub(super) fn update(&mut self) -> Result<(), Chip8Error>{
        // Draw a grid
        self.update_buffer();
        
        // Update the window with buffer
        self.window.as_mut().unwrap()
            .update_with_buffer(&self.buffer, DISPLAY_WIDTH, DISPLAY_HEIGHT)
            .map_err(Chip8Error::WindowUpdateError)
    }

    pub(super) fn clear(&mut self) {
        self.grid = [[false; DISPLAY_HEIGHT]; DISPLAY_WIDTH];
    }

    pub(super) fn draw(&mut self, x: usize, y: usize, sprite: impl Iterator<Item = u8>) -> bool {
        let mut collision = false;
        for (j, byte) in sprite.enumerate() {
            for i in 0..8 {
                let xi = (x + i) % DISPLAY_WIDTH;
                let yj = (y + j) % DISPLAY_HEIGHT;
                let old = self.grid[xi][yj];
                let new = (byte & (0x80 >> i)) != 0;
                self.grid[xi][yj] ^= new;
                collision |= old && !self.grid[xi][yj];
            }
        }
        collision
    }

    // Update buffer with grid
    fn update_buffer(&mut self) {
        for i in 0..DISPLAY_WIDTH {
            for j in 0..DISPLAY_HEIGHT {
                let color = if self.grid[i][j] { self.colors.filled } else { self.colors.empty };
                self.buffer[i + j * DISPLAY_WIDTH] = color;
            }
        }
    }
}
