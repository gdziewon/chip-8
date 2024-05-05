mod sprites;

use std::io::{BufRead, BufReader};
use std::fs::File;
use anyhow::{Result, anyhow};

const MEMORY_SIZE: usize = 1024 * 4;

pub struct Memory {
    memory: [u8; MEMORY_SIZE]
}

impl Memory {
    pub fn new() -> Self {
        let mut memory = [0; MEMORY_SIZE];

        sprites::set_character_sprites(&mut memory);

        Memory { memory }
    }

    pub fn get_byte(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    pub fn get_2bytes(&self, addr: u16) -> u16 {
        ((self.get_byte(addr) as u16) << 8)  | self.get_byte(addr + 1) as u16
    }

    pub fn write_byte(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }

    pub fn write_2bytes(&mut self, addr: u16, data: u16) {
        self.write_byte(addr, (data >> 8) as u8);
        self.write_byte(addr + 1, (data & 0xff) as u8);
    }

    pub fn write_entry(&mut self, entry: MemoryEntry) {
        self.write_byte(entry.address, entry.data);
    }

    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Self> {
        args.next();

        match args.next() {
            Some(file_path) => Memory::from(&file_path),
            None => Err(anyhow!("File for memory not specified")),
        }
    }

    pub fn from(file_path: &str) -> Result<Self> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut memory = Self::new();

        for (i, line) in reader.lines().enumerate() {
            let line = line?;
            let entry = MemoryEntry::from_line(&line, i + 1)?;
            memory.write_entry(entry);
        }

        Ok(memory)
    }
}

pub struct MemoryEntry {
    address: u16,
    data: u8,
}

impl MemoryEntry {
    fn from_line(line: &str, line_number: usize) -> Result<Self> {
        let mut parts = line.split_whitespace();
        let addr_str = parts.next().ok_or_else(|| anyhow!("Missing address in line {}", line_number))?;
        let data_str = parts.next().ok_or_else(|| anyhow!("Missing data in line {}", line_number))?;

        let address = u16::from_str_radix(addr_str.trim_start_matches("0x"), 16)
            .map_err(|e| anyhow!("Invalid address at line {}: {}", line_number, e))?;
        let data = u8::from_str_radix(data_str.trim_start_matches("0x"), 16)
            .map_err(|e| anyhow!("Invalid data at line {}: {}", line_number, e))?;

        Ok(MemoryEntry { address, data })
    }
}