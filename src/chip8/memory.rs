use std::error::Error;
use std::io::{BufReader, Read};
use std::fs::File;
use super::{MEMORY_SIZE, PROGRAM_START, errors::Chip8Error};

pub struct Memory {
    memory: [u8; MEMORY_SIZE]
}

impl Memory {
    pub fn new() -> Self {
        let mut memory = [0; MEMORY_SIZE];

        // Font sprites
        let sprites = [
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

        // Load font sprites into memory - 0x00 to 0x4F
        for (i, &byte) in sprites.iter().enumerate() {
            memory[i] = byte;
        }

        Memory { memory }
    }

    // Assumes addr is always valid, panics if out of bounds
    pub fn read_byte(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }
    
    // Same here
    pub fn write_byte(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }

    // Fetches an instruction from memory - 2 bytes
    pub fn get_instruction(&self, addr: u16) -> u16 {
        let high_byte = self.read_byte(addr);
        let low_byte = self.read_byte(addr + 1);
    
        ((high_byte as u16) << 8) | low_byte as u16
    }

    // Loads program from file
    pub fn load(&mut self, file: &File) -> Result<(), Box<dyn Error>> {
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

    // Loads file from args - 2nd argument
    pub fn from_args(mut args: impl Iterator<Item = String>) -> Result<Memory, Box<dyn Error>> {
        match (args.next(), args.next()) {
            (Some(_), Some(file_path)) => {
                let mut memory = Memory::new();
                memory.load(&File::open(file_path)?)?;
                Ok(memory)
            },
            _ => Err(Box::new(Chip8Error::MissingFilePath))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let memory = Memory::new();
        assert_eq!(memory.read_byte(0), 0xF0);
        assert_eq!(memory.read_byte(1), 0x90);
        assert_eq!(memory.read_byte(2), 0x90);
        assert_eq!(memory.read_byte(3), 0x90);
        assert_eq!(memory.read_byte(4), 0xF0);
    }

    #[test]
    fn test_read_write_byte() {
        let mut memory = Memory::new();
        memory.write_byte(0x200, 0xAB);
        assert_eq!(memory.read_byte(0x200), 0xAB);
    }

    #[test]
    #[should_panic]
    fn test_read_byte_out_of_bounds() {
        let memory = Memory::new();
        memory.read_byte(MEMORY_SIZE as u16);
    }

    #[test]
    #[should_panic]
    fn test_write_byte_out_of_bounds() {
        let mut memory = Memory::new();
        memory.write_byte(MEMORY_SIZE as u16, 0xAB);
    }

    #[test]
    fn test_get_instruction() {
        let mut memory = Memory::new();
        memory.write_byte(0x200, 0xAB);
        memory.write_byte(0x201, 0xCD);
        assert_eq!(memory.get_instruction(0x200), 0xABCD);
    }
}