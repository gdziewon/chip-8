use std::{fmt, error};

#[derive(Debug)]
pub enum Chip8Error {
    FileNotFound(String),
    PermissionDenied(String),
    FileReadError(String),
    MissingFilePath,
    MissingAddress(usize),
    InvalidAddress(usize, String),
    MissingData(usize),
    InvalidData(usize, String),
    MemoryEntryError(usize),
    AddressTooHigh(u16, usize),
    AddressTooLow(u16, u16),
    TooManyLines(usize, usize),
    RegisterIndexOutOfBounds(usize),
    UnrecognizedOpcode(u16),
    FlagRegisterUsed,
    WindowCreationError(minifb::Error),
    WindowUpdateError(minifb::Error),
}

impl fmt::Display for Chip8Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Chip8Error::FileNotFound(file_path) => write!(f, "File not found: {}", file_path),
            Chip8Error::PermissionDenied(file_path) => write!(f, "Permission denied: {}", file_path),
            Chip8Error::FileReadError(file_path) => write!(f, "Failed to read file: {}", file_path),
            Chip8Error::MissingFilePath => write!(f, "Expected a file path as the argument"),
            Chip8Error::MissingAddress(line) => write!(f, "Missing address in line {}", line),
            Chip8Error::InvalidAddress(line, addr) => write!(f, "Invalid address at line {}: {}", line, addr),
            Chip8Error::MissingData(line) => write!(f, "Missing data in line {}", line),
            Chip8Error::InvalidData(line, data) => write!(f, "Invalid data at line {}: {}", line, data),
            Chip8Error::MemoryEntryError(line) => write!(f, "Error at line {}", line),
            Chip8Error::AddressTooHigh(address, size) => write!(f, "Address too high: {:#X}, memory size is {:#X}", address, size),
            Chip8Error::AddressTooLow(address, start) => write!(f, "Address too low: {:#X}, program starts at {:#X}", address, start),
            Chip8Error::TooManyLines(lines, available) => write!(f, "File has too many lines: {}. Maximum memory available for a program is {}.", lines, available),
            Chip8Error::RegisterIndexOutOfBounds(index) => write!(f, "Register index out of bounds: {}", index),
            Chip8Error::UnrecognizedOpcode(op) => write!(f, "Unrecognized opcode: {:#X}", op),
            Chip8Error::FlagRegisterUsed => write!(f, "Flag register shouldn't be used directly"),
            Chip8Error::WindowCreationError(e) => write!(f, "Window creation error: {}", e),
            Chip8Error::WindowUpdateError(e) => write!(f, "Window update error: {}", e),
        }
    }
}

impl error::Error for Chip8Error {}
