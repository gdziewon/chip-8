use std::io::{BufRead, BufReader};
use std::fs::File;

pub struct Memory {
    memory: [u8; 1024 * 4]
}

impl Memory {
    pub fn new() -> Self {
        let mut memory: [u8; 1024 * 4] = [0; 1024 * 4];

        // "0"
        memory[0x00] = 0xf0;
        memory[0x01] = 0x90;
        memory[0x02] = 0x90;
        memory[0x03] = 0x90;
        memory[0x04] = 0xf0;
        
        // "1"
        memory[0x05] = 0x20;
        memory[0x06] = 0x60;
        memory[0x07] = 0x20;
        memory[0x08] = 0x20;
        memory[0x09] = 0x70;

        // "2"
        memory[0x0a] = 0xf0;
        memory[0x0b] = 0x10;
        memory[0x0c] = 0xf0;
        memory[0x0d] = 0x80;
        memory[0x0e] = 0xf0;
        // "3"
        memory[0x0f] = 0xf0;
        memory[0x10] = 0x10;
        memory[0x11] = 0xf0;
        memory[0x12] = 0x10;
        memory[0x13] = 0xf0;
        // "4"
        memory[0x14] = 0x90;
        memory[0x15] = 0x90;
        memory[0x16] = 0xf0;
        memory[0x17] = 0x10;
        memory[0x18] = 0x10;
        // "5"
        memory[0x19] = 0xf0;
        memory[0x1a] = 0x80;
        memory[0x1b] = 0xf0;
        memory[0x1c] = 0x10;
        memory[0x1d] = 0xf0;
        // "6"
        memory[0x1e] = 0xf0;
        memory[0x1f] = 0x80;
        memory[0x20] = 0xf0;
        memory[0x21] = 0x90;
        memory[0x22] = 0xf0;
        // "7"
        memory[0x23] = 0xf0;
        memory[0x24] = 0x10;
        memory[0x25] = 0x20;
        memory[0x26] = 0x40;
        memory[0x27] = 0x40;
        // "8"
        memory[0x28] = 0xf0;
        memory[0x29] = 0x90;
        memory[0x2a] = 0xf0;
        memory[0x2b] = 0x90;
        memory[0x2c] = 0xf0;
        // "9"
        memory[0x2d] = 0xf0;
        memory[0x2e] = 0x90;
        memory[0x2f] = 0xf0;
        memory[0x30] = 0x10;
        memory[0x31] = 0xf0;
        // "A"
        memory[0x32] = 0xf0;
        memory[0x33] = 0x90;
        memory[0x34] = 0xf0;
        memory[0x35] = 0x90;
        memory[0x36] = 0x90;
        // "B"
        memory[0x37] = 0xe0;
        memory[0x38] = 0x90;
        memory[0x39] = 0xe0;
        memory[0x3a] = 0x90;
        memory[0x3b] = 0xe0;
        // "C"
        memory[0x3c] = 0xf0;
        memory[0x3d] = 0x80;
        memory[0x3e] = 0x80;
        memory[0x3f] = 0x80;
        memory[0x40] = 0xf0;
        // "D"
        memory[0x41] = 0xe0;
        memory[0x42] = 0x90;
        memory[0x43] = 0x90;
        memory[0x44] = 0x90;
        memory[0x45] = 0xe0;
        // "E"
        memory[0x46] = 0xf0;
        memory[0x47] = 0x80;
        memory[0x48] = 0xf0;
        memory[0x49] = 0x80;
        memory[0x4a] = 0xf0;
        // "F"
        memory[0x4b] = 0xf0;
        memory[0x4c] = 0x80;
        memory[0x4d] = 0xf0;
        memory[0x4e] = 0x80;
        memory[0x4f] = 0x80;

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
            panic!("Can't write to memory addresses 0x00 - 0x200");
        }
        self.memory[addr as usize] = data;
    }

    pub fn write_2bytes(&mut self, addr: u16, data: u16) {
        self.write_byte(addr, (data >> 8) as u8);
        self.write_byte(addr + 1, (data & 0xff) as u8);
    }

    pub fn from(file_path: &str) -> Result<Self, std::io::Error> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut memory = Self::new();

        for line in reader.lines() {
            let line = line?;
            let mut split = line.split_whitespace();

            let addr = split.next().ok_or_else(|| std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Missing address in file"
            ))?;
            let data = split.next().ok_or_else(|| std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Missing data in file"
            ))?;

            let addr = u16::from_str_radix(addr.trim_start_matches("0x"), 16).map_err(|e| {
                println!("Failed to parse address: {}", addr); // Debug print
                std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
            })?;
            let data = u8::from_str_radix(data.trim_start_matches("0x"), 16).map_err(|e| {
                println!("Failed to parse data: {}", data); // Debug print
                std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
            })?;

            memory.write_byte(addr, data);
        }

        Ok(memory)
    }
}
