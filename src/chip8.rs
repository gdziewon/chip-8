pub mod memory;
pub mod cpu;
pub mod io;

#[cfg(test)]
mod tests;

pub use memory::Memory;
use crate::{chip8::io::Color, errors::Chip8Error};
use cpu::CPU;

use std::{collections::HashMap, fs::File, thread, time::{Duration, Instant}};

use minifb::{Key, Scale}; // GUI library

use crate::chip8::io::IO; // Audio library

// Memory
pub const MEMORY_SIZE: usize = 1024 * 4;
pub const PROGRAM_START: u16 = 0x200;

// Display and timers update frequency
pub const CPU_FREQ: f64 = 1.0 / 700.0; // 500hz

pub struct Chip8 {
    cpu: CPU,
    mem: Memory,
    io: IO
}

impl Chip8 {
    // Creates a new Chip8 instance with the given key bindings
    pub fn new() -> Self {
        let cpu = CPU::new();
        let mem = Memory::new();
        let io = IO::new();
        Chip8 { cpu, mem, io }
    }

    pub fn load_program(&mut self, file: &File) -> Result<(), Box<dyn std::error::Error>> {
        self.mem.load_from_file(file)
    }

    pub fn run(&mut self) -> Result<(), Chip8Error> {
        // Open window
        self.io.display_init()?;

        let tick = Duration::from_secs_f64(CPU_FREQ);
        let mut next = Instant::now() + tick;

        while self.io.display_is_open() {
            let now = Instant::now();
            if now >= next {
                self.cpu.execute(&mut self.mem, &mut self.io)?;

                self.io.display_update()?;

                if self.cpu.sound_timer() > 0 {
                    self.io.audio_play();
                } else {
                    self.io.audio_pause();
                }

                next += tick
            } else {
                thread::sleep(next - now);
            }
        }

        self.cpu.shutdown(); // FIXME: when clicking X, CPU stops but the window doesnt close

        Ok(())
    }

    pub fn set_colors(&mut self, filled: Color, empty: Color) {
        self.io.display_set_colors(filled, empty);
    }

    pub fn set_keyboard_bindings(&mut self, bindings: HashMap<u8, Key>) {
        self.io.keyboard_set_bindings(bindings);
    }

    pub fn set_scale(&mut self, scale: Scale) {
        self.io.display_set_scale(scale);
    }
}

