use minifb::{Key, KeyRepeat, Scale, Window, WindowOptions};
use super::{DISPLAY_HEIGHT, DISPLAY_WIDTH, DISPLAY_SCALE, WINDOW_NAME};
use super::errors::Chip8Error;

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
            filled: 0xffffff,
            empty: 0x000000
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
    pub fn get_key_press(&mut self, keyboard: &super::Keys) -> Option<u8> {
        self.window.as_ref().unwrap().get_keys_pressed(KeyRepeat::No)
        .iter()
        .find_map(|&k| keyboard.get_by_key(&k))
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
    pub(super) fn set_colors(&mut self, filled: u32, empty: u32) {
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
                let color = if self.grid[i][j] { self.colors.filled } else { self.colors.empty };
                self.buffer[i + j * DISPLAY_WIDTH] = color;
            }
        }
    }


}

struct Colors {
    filled: u32,
    empty: u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use minifb::Scale;

    #[test]
    fn test_draw() {
        let mut display = Display::new();
        let sprite = vec![0b10000000, 0b01000000, 0b00100000, 0b00010000, 0b00001000];
        let collision = display.draw(0, 0, sprite.iter().copied());
        assert_eq!(collision, false);
        assert_eq!(display.grid[0][0], true);
        assert_eq!(display.grid[1][1], true);
        assert_eq!(display.grid[2][2], true);
        assert_eq!(display.grid[3][3], true);
        assert_eq!(display.grid[4][4], true);
    }

    #[test]
    fn test_clear() {
        let mut display = Display::new();
        let sprite = vec![0b10000000, 0b01000000, 0b00100000, 0b00010000, 0b00001000];
        display.draw(0, 0, sprite.iter().copied());
        display.clear();
        for i in 0..DISPLAY_WIDTH {
            for j in 0..DISPLAY_HEIGHT {
                assert_eq!(display.grid[i][j], false);
            }
        }
    }

    #[test]
    fn test_update_buffer() {
        let mut display = Display::new();
        let sprite = vec![0b10000000, 0b01000000, 0b00100000, 0b00010000, 0b00001000];
        display.draw(0, 0, sprite.iter().copied());
        display.update_buffer();
        assert_eq!(display.buffer[0], display.colors.filled);
        assert_eq!(display.buffer[1 + DISPLAY_WIDTH], display.colors.filled);
        assert_eq!(display.buffer[2 + 2 * DISPLAY_WIDTH], display.colors.filled);
        assert_eq!(display.buffer[3 + 3 * DISPLAY_WIDTH], display.colors.filled);
        assert_eq!(display.buffer[4 + 4 * DISPLAY_WIDTH], display.colors.filled);
    }

    #[test]
    fn test_set_colors() {
        let mut display = Display::new();
        display.set_colors(0x123456, 0x654321);
        assert_eq!(display.colors.filled, 0x123456);
        assert_eq!(display.colors.empty, 0x654321);
    }

    #[test]
    fn test_set_scale() {
        let mut display = Display::new();
        display.set_scale(Scale::X2);
        assert_eq!(display.scale as u32, Scale::X2 as u32);
    }

    #[test]
    fn test_init() {
        let mut display = Display::new();
        display.init().unwrap();
        assert!(display.window.is_some());
        assert!(display.window.as_ref().unwrap().is_open());
        assert!(display.is_open());
    }

    #[test]
    fn test_close() {
        let mut display = Display::new();
        display.init().unwrap();
        display.close();
        assert!(display.window.is_none());
        assert!(!display.is_open());
    }
}