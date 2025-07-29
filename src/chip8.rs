pub mod memory;
pub mod cpu;
pub mod io;

#[cfg(test)]
mod tests;

pub use memory::Memory;
use crate::errors::Chip8Error;
use cpu::CPU;

use std::{collections::HashMap, fs::File, thread, time::{Duration, Instant}};

use minifb::{Key, Scale}; // GUI library

use crate::chip8::io::IO; // Audio library

// Memory
pub const MEMORY_SIZE: usize = 1024 * 4;
pub const PROGRAM_START: u16 = 0x200;

// Delay between each instruction execution
const MS_DELAY: u64 = 1; // todo: figure out the CPU frequency

// Display and timers update frequency
pub const DISPLAY_AND_TIMERS_UPDATE_FREQUENCY: u64 = 1000 / 60; // 60hz

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

        let mut last_update = Instant::now();

        while self.io.display_is_open() {
            // Execute instruction
            self.cpu.execute(&mut self.mem, &mut self.io)?;

            // Delay between each instruction for more accurate timing
            thread::sleep(Duration::from_millis(MS_DELAY));

            // Update timers and display at 60hz
            if last_update.elapsed() >= Duration::from_millis(DISPLAY_AND_TIMERS_UPDATE_FREQUENCY) {
                self.io.update(self.cpu.sound_timer());
                last_update = Instant::now();
            }
        }
        Ok(())
    }

    pub fn set_colors(&mut self, filled: u32, empty: u32) {
        self.io.display_set_colors(filled, empty);
    }

    pub fn set_keyboard_bindings(&mut self, bindings: HashMap<u8, Key>) {
        self.io.keyboard_set_bindings(bindings);
    }

    pub fn set_scale(&mut self, scale: Scale) {
        self.io.display_set_scale(scale);
    }
}

