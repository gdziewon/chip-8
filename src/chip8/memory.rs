mod sprites;

use std::{fs, io::ErrorKind};
use super::{MEMORY_SIZE, PROGRAM_START, errors::Chip8Error};

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

    pub fn read_byte(&self, addr: MemoryAddress) -> u8 {
        self.memory[addr.get() as usize]  // Direct access since MemoryAddress is already validated
    }
    

    pub fn get_instruction(&self, addr: u16) -> Result<u16, Chip8Error> {
        let high_byte = self.read_byte(MemoryAddress::new(addr)?);
        let low_byte = self.read_byte(MemoryAddress::new(addr + 1)?);
    
        Ok(((high_byte as u16) << 8) | low_byte as u16)
    }
    

    pub fn write_entry(&mut self, entry: MemoryEntry) {
        self.memory[entry.get_address() as usize] = entry.data;
    }
    

    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Self, Chip8Error> {
        match (args.next(), args.next()) {
            (Some(_), Some(file_path)) => {
                match fs::read_to_string(&file_path) {
                    Ok(contents) => Memory::from(&contents),
                    Err(err) => match err.kind() {
                        ErrorKind::NotFound => Err(Chip8Error::FileNotFound(file_path.to_string())),
                        ErrorKind::PermissionDenied => Err(Chip8Error::PermissionDenied(file_path.to_string())),
                        _ => Err(Chip8Error::FileReadError(file_path.to_string())),
                    },
                }
            },
            _ => Err(Chip8Error::MissingFilePath),
        }
    }

    pub fn from(contents: &str) -> Result<Self, Chip8Error> {
        let lines: Vec<_> = contents.lines().collect();
        if lines.len() > AVAILABLE_MEMORY {
            return Err(Chip8Error::TooManyLines(lines.len(), AVAILABLE_MEMORY));
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
    address: MemoryAddress,
    data: u8,
}

impl MemoryEntry {
    pub fn new(address: u16, data: u8) -> Result<Self, Chip8Error> {
        let memory_address = MemoryAddress::new(address)?;
        if address < PROGRAM_START {
            Err(Chip8Error::AddressTooLow(address, PROGRAM_START))
        } else {
            Ok(MemoryEntry { address: memory_address, data })
        }
    }

    fn from_line(line: &str, line_number: usize) -> Result<Self, Chip8Error> {
        let mut parts = line.split_whitespace();
        
        let address = parts.next()
            .ok_or(Chip8Error::MissingAddress(line_number))
            .and_then(|addr| u16::from_str_radix(addr.strip_prefix("0x").unwrap_or(addr), 16)
                .map_err(|_| Chip8Error::InvalidAddress(line_number, addr.to_string())))?;
        
        let data = parts.next()
            .ok_or(Chip8Error::MissingData(line_number))
            .and_then(|data| u8::from_str_radix(data.strip_prefix("0x").unwrap_or(data), 16)
                .map_err(|_| Chip8Error::InvalidData(line_number, data.to_string())))?;
        
        MemoryEntry::new(address, data).map_err(|_| Chip8Error::MemoryEntryError(line_number))
    }

    pub fn get_address(&self) -> u16 {
        self.address.get()
    }
    
}

pub struct MemoryAddress {
    address: u16,
}

impl MemoryAddress {
    pub fn new(address: u16) -> Result<Self, Chip8Error> {
        if address >= MEMORY_SIZE as u16 {
            Err(Chip8Error::AddressTooHigh(address, MEMORY_SIZE))
        } else {
            Ok(MemoryAddress { address })
        }
    }

    pub fn get(&self) -> u16 {
        self.address
    }
}
