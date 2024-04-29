use std::io::{BufRead, BufReader};
use std::fs::File;
mod sprites;

pub struct Memory {
    memory: [u8; 1024 * 4]
}

impl Memory {
    pub fn new() -> Self {
        let mut memory: [u8; 1024 * 4] = [0; 1024 * 4];

        sprites::set_character_sprites(&mut memory);

        Memory {
            memory
        }
    }

    pub fn get_byte(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    pub fn get_2bytes(&self, addr: u16) -> u16 {
        ((self.get_byte(addr) as u16) << 8)  | self.get_byte(addr + 1) as u16
    }

    pub fn write_byte(&mut self, addr: u16, data: u8) {
        if addr < 0x200 {
            panic!("Can't write to memory addresses 0x000 - 0x200");
        }
        self.memory[addr as usize] = data;
    }

    pub fn write_2bytes(&mut self, addr: u16, data: u16) {
        self.write_byte(addr, (data >> 8) as u8);
        self.write_byte(addr + 1, (data & 0xff) as u8);
    }

    // Loading memory from file, pairs "addr data" need to be written line by line in hex, seperated by space
    pub fn from(file_path: &str) -> Result<Self, std::io::Error> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut memory = Self::new();

        for (i, line) in reader.lines().enumerate() {
            let line = line?;
            let mut split = line.split_whitespace();

            // Get addr and data
            let addr = split.next().ok_or_else(|| std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Missing address in line {}", i+1)
            ))?;
            let data = split.next().ok_or_else(|| std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Missing data in line {}", i+1)
            ))?;

            // Convert values to u16 and u8
            let addr = u16::from_str_radix(addr.trim_start_matches("0x"), 16).map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
            })?;
            let data = u8::from_str_radix(data.trim_start_matches("0x"), 16).map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
            })?;

            // Write byte to memory
            memory.write_byte(addr, data);
        }

        Ok(memory)
    }
}