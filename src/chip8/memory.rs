mod sprites;

use std::fs;
use anyhow::{Result, Context, anyhow};
use super::{MEMORY_SIZE, PROGRAM_START};

const AVAILABLE_MEMORY: usize = MEMORY_SIZE - PROGRAM_START as usize;

pub struct Memory {
    memory: [u8; MEMORY_SIZE]
}

impl Memory {
    pub fn new() -> Self {
        let mut memory = [0; MEMORY_SIZE];

        sprites::set_sprites(&mut memory);

        Memory { memory }
    }

    // TODO: Make them return Result
    pub fn get_byte(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    pub fn get_instruction(&self, addr: u16) -> u16 {
        ((self.get_byte(addr) as u16) << 8)  | self.get_byte(addr + 1) as u16
    }

    pub fn write_entry(&mut self, entry: MemoryEntry) {
        self.memory[entry.address as usize] = entry.data;
    }
    

    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Self> {
        match (args.next(), args.next()) {
            (Some(_), Some(file_path)) => {
                let contents = fs::read_to_string(&file_path)
                    .with_context(|| format!("Failed to read file: {}", file_path))?;
                Memory::from(&contents)
            },
            _ => Err(anyhow!("Expected a file path as the second argument")),
        }
    }

    pub fn from(contents: &str) -> Result<Self> {
        let lines: Vec<_> = contents.lines().collect();
        if lines.len() > AVAILABLE_MEMORY {
            return Err(anyhow!("File has too many lines: {}. Maximum memory available for a program is {}.", lines.len(), AVAILABLE_MEMORY));
        }
    
        let mut memory = Self::new();
        
        for (i, line) in lines.iter().enumerate() {
            let entry = MemoryEntry::from_line(line, i + 1)?;
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
    pub fn new(address: u16, data: u8) -> Result<Self> {
        match address {
            addr if addr >= MEMORY_SIZE as u16 => Err(anyhow!("Address too high: {}, memory size is {}", addr, MEMORY_SIZE)),
            addr if addr < PROGRAM_START => Err(anyhow!("Address too low: {}, program starts at {}", addr, PROGRAM_START)),
            _ => Ok(MemoryEntry { address, data }),
        }
    }

    fn from_line(line: &str, line_number: usize) -> Result<Self> {
        let mut parts = line.split_whitespace();
        
        let address = parts.next()
            .ok_or_else(|| anyhow!("Missing address in line {}", line_number))
            .and_then(|addr| u16::from_str_radix(addr.strip_prefix("0x").unwrap_or(addr), 16)
                .map_err(|_| anyhow!("Invalid address at line {}: {}", line_number, addr)))?;
        
        let data = parts.next()
            .ok_or_else(|| anyhow!("Missing data in line {}", line_number))
            .and_then(|data| u8::from_str_radix(data.strip_prefix("0x").unwrap_or(data), 16)
                .map_err(|_| anyhow!("Invalid data at line {}: {}", line_number, data)))?;
        
        MemoryEntry::new(address, data).map_err(|err| anyhow!("Error at line {}: {}", line_number, err))
    }
    
}