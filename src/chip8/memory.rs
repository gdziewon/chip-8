use std::error::Error;
use std::io::{BufReader, Read};
use std::fs::File;


use crate::chip8::cpu::Addr;
use crate::errors::Chip8Error;

pub const MEMORY_SIZE: usize = 1024 * 4;
pub const PROGRAM_START: u16 = 0x200;
const SPRITES_MEMORY: usize = 80;

const SPRITES: [u8; SPRITES_MEMORY] = [
            0xf0, 0x90, 0x90, 0x90, 0xf0, // "0"
            0x20, 0x60, 0x20, 0x20, 0x70, // "1"
            0xf0, 0x10, 0xf0, 0x80, 0xf0, // "2"
            0xf0, 0x10, 0xf0, 0x10, 0xf0, // "3"
            0x90, 0x90, 0xf0, 0x10, 0x10, // "4"
            0xf0, 0x80, 0xf0, 0x10, 0xf0, // "5"
            0xf0, 0x80, 0xf0, 0x90, 0xf0, // "6"
            0xf0, 0x10, 0x20, 0x40, 0x40, // "7"
            0xf0, 0x90, 0xf0, 0x90, 0xf0, // "8"
            0xf0, 0x90, 0xf0, 0x10, 0xf0, // "9"
            0xf0, 0x90, 0xf0, 0x90, 0x90, // "A"
            0xe0, 0x90, 0xe0, 0x90, 0xe0, // "B"
            0xf0, 0x80, 0x80, 0x80, 0xf0, // "C"
            0xe0, 0x90, 0x90, 0x90, 0xe0, // "D"
            0xf0, 0x80, 0xf0, 0x80, 0xf0, // "E"
            0xf0, 0x80, 0xf0, 0x80, 0x80  // "F"
];

pub struct Memory {
    memory: [u8; MEMORY_SIZE]
}


impl Memory {
    pub fn new() -> Self {
        let mut memory = [0; MEMORY_SIZE];

        // Load font sprites - 0x00 to 0x4F
        for (i, &byte) in SPRITES.iter().enumerate() {
            memory[i] = byte;
        }

        Memory { memory }
    }

    pub fn read_byte(&self, addr: Addr) -> u8 {
        self.memory[addr.value() as usize]
    }

    pub fn write_byte(&mut self, addr: Addr, data: u8) {
        self.memory[addr.value() as usize] = data;
    }

    // Fetches an instruction from memory - 2 bytes
    pub fn get_instruction(&self, addr: Addr) -> u16 {
        let high_byte = self.read_byte(addr);
        let low_byte = self.read_byte(addr + 1);

        ((high_byte as u16) << 8) | low_byte as u16
    }

    pub fn load(&mut self, program: impl Iterator<Item = u8>) -> Result<(), Box<dyn Error>> {
        for (i, byte) in program.enumerate() {
            let idx = PROGRAM_START as usize + i;
            if idx >= MEMORY_SIZE {
                return Err(Box::new(Chip8Error::TooManyLines(i, MEMORY_SIZE)));
            }
            self.memory[idx] = byte;
        }
        Ok(())
    }

    // Loads program from file
    pub fn load_from_file(&mut self, file: &File) -> Result<(), Box<dyn Error>> {
        let f = BufReader::new(file);

        for (i, byte) in f.bytes().enumerate() {
            let idx = PROGRAM_START as usize + i;
            if idx >= MEMORY_SIZE {
                return Err(Box::new(Chip8Error::TooManyLines(i, MEMORY_SIZE)));
            }
            self.memory[idx] = byte?;
        }
        Ok(())
    }
}