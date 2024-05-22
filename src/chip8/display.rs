use minifb::{Window, WindowOptions};
use super::{DISPLAY_HEIGHT, DISPLAY_WIDTH, DISPLAY_SCALE, errors::Chip8Error};

pub struct Display {
    pub window: Window,
    buffer: Vec<u32>
}

impl Display {
    pub fn new() -> Result<Self, Chip8Error> {
        let buffer: Vec<u32> = vec![0; DISPLAY_WIDTH * DISPLAY_HEIGHT * DISPLAY_SCALE * DISPLAY_SCALE];
        let window = Window::new(
            "Chip8",
            DISPLAY_WIDTH * DISPLAY_SCALE,
            DISPLAY_HEIGHT * DISPLAY_SCALE,
            WindowOptions::default(),
        )
        .map_err(Chip8Error::WindowCreationError)?;
    
        Ok(Display { buffer, window })
    }
    
    pub fn update(&mut self, display: &[[bool; DISPLAY_HEIGHT]; DISPLAY_WIDTH]) -> Result<(), Chip8Error>{
        // Draw a grid
        self.update_buffer(&display);
    
        // Update the window with buffer
        self.window
            .update_with_buffer(&self.buffer, DISPLAY_WIDTH * DISPLAY_SCALE, DISPLAY_HEIGHT * DISPLAY_SCALE)
            .map_err(Chip8Error::WindowUpdateError)
    }


    fn update_buffer(&mut self, display: &[[bool; DISPLAY_HEIGHT]; DISPLAY_WIDTH]) {
        for i in 0..DISPLAY_WIDTH {
            for j in 0..DISPLAY_HEIGHT {
                let color = if display[i][j] { 0xffffff } else { 0x0000000 };
                self.fill_square( i, j, color);
            }
        }
    }
    
    fn fill_square(&mut self, x: usize, y: usize, color: u32) {
        for i in 0..DISPLAY_SCALE {
            for j in 0..DISPLAY_SCALE {
                self.buffer[(x * DISPLAY_SCALE + i) + ((y * DISPLAY_SCALE + j) * DISPLAY_WIDTH * DISPLAY_SCALE)] = color;
            }
        }
    }
}
