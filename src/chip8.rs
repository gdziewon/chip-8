mod display;
mod keys;
pub mod memory;
pub mod errors;

#[cfg(test)]
mod tests;

pub use memory::Memory;
use errors::Chip8Error;
use display::Display;
use keys::Keys;

use std::{collections::HashMap, thread, time::{Duration, Instant}};

use rand;
use minifb::{Key, Scale}; // GUI library
use rodio::{OutputStream, Sink, source::{SineWave, Source}}; // Audio library

// Display
pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;
pub const DISPLAY_SCALE: Scale = Scale::X16;
const WINDOW_NAME: &str = "Chip8 Emulator";

// Memory
pub const MEMORY_SIZE: usize = 1024 * 4;
pub const PROGRAM_START: u16 = 0x200;
const SPRITE_SIZE: u16 = 5;

// Chip8 specifications
const NUM_REGISTERS: usize = 16;
const FLAG_REGISTER: usize = 15;
const STACK_DEPTH: usize = 16;

// Sound
const SINEWAVE_FREQUENCY: f32 = 440.0; // A4

// Delay between each instruction execution
const MS_DELAY: u64 = 1;

// Display and timers update frequency
pub const DISPLAY_AND_TIMERS_UPDATE_FREQUENCY: u64 = 1000 / 60; // 60hz

pub struct Chip8 {
    // Registers
    v: [u8; NUM_REGISTERS], // 16 general purpose 8-bit registers
    idx: u16, // 16-bit address register

    // Timers - counts down at 60hz to 0
    dt: u8, // delay timer
    st: u8, // sound timer

    pc: u16, // Program counter
    sp: u8, // Stack pointer
    stack: [u16; STACK_DEPTH], // 16 16-bit stack fields

    display: Display, // Display struct 

    keyboard: Keys, // Key bindings

    audio: Sink, // Audio sink
}


impl Chip8 {
    // Creates a new Chip8 instance with the given key bindings
    pub fn new() -> Self {
        // Key bindings setup
        let keyboard = Keys::get_default();

        // Display setup
        let display = Display::new();

        // Audio setup
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let audio = Sink::try_new(&stream_handle).unwrap();
        let source = SineWave::new(SINEWAVE_FREQUENCY).repeat_infinite();
        audio.append(source);
        audio.pause();

        Chip8 {
            v: [0x00; NUM_REGISTERS],
            idx: 0x0000,
            dt: 0,
            st: 0,
            pc: PROGRAM_START,
            sp: 0x00,
            stack: [0x0000; STACK_DEPTH],
            display,
            keyboard,
            audio
        }
    }

    pub fn run( &mut self, mem: &mut Memory ) -> Result<(), Chip8Error> {
        // Open window
        self.display.init()?;

        let mut last_update = Instant::now(); 

        while self.display.is_open() {
            // Fetch instruction
            let instruction: u16 = mem.get_instruction(self.pc);

            // Increment program counter
            self.pc += 2; 

            // Execute instruction
            self.execute(instruction, mem)?; 

            // Delay between each instruction for more accurate timing
            thread::sleep(Duration::from_millis(MS_DELAY)); 
            
            // Update timers and display at 60hz
            if last_update.elapsed() >= Duration::from_millis(DISPLAY_AND_TIMERS_UPDATE_FREQUENCY) {
                self.display.update()?;
                self.update_timers();
                last_update = Instant::now();
            }
        }
        Ok(())
    }

    fn update_timers(&mut self) {
        if self.st > 0 { // Decrement sound timer at 60hz
            self.audio.play(); // Play sound when sound timer is greater than 0
            self.st -= 1;
        } else {
            self.audio.pause(); // Pause sound when sound timer is 0
        }

        if self.dt > 0 { // Decrement delay timer at 60hz
            self.dt -= 1;
        }
    }

    // Executes given opcode dividing them by their first nibble
    fn execute( &mut self, op_code: u16, mem: &mut Memory) -> Result<(), Chip8Error> {
        let op_code = OpCode::new(op_code); // Create OpCode struct for easier access
        match op_code.code >> 12 {
            0x0 => self.execute_0nnn(op_code)?,
            0x1 => self.execute_1nnn(op_code),
            0x2 => self.execute_2nnn(op_code),
            0x3 => self.execute_3xkk(op_code),
            0x4 => self.execute_4xkk(op_code),
            0x5 => self.execute_5xy0(op_code)?,
            0x6 => self.execute_6xkk(op_code),
            0x7 => self.execute_7xkk(op_code),
            0x8 => self.execute_8nnn(op_code)?,
            0x9 => self.execute_9xy0(op_code)?,
            0xA => self.execute_annn(op_code),
            0xB => self.execute_bnnn(op_code),
            0xC => self.execute_cxkk(op_code),
            0xD => self.execute_dxyn(op_code, &mem),
            0xE => self.execute_ennn(op_code)?,
            0xF => self.execute_fnnn(op_code, mem)?,
            _ => return Err(Chip8Error::UnrecognizedOpcode(op_code.code, self.pc - 2)), // Impossible to reach
        }
        Ok(())
    }

    // 0x0nnn - System calls
    fn execute_0nnn( &mut self, op_code: OpCode) -> Result<(), Chip8Error>{
        match op_code.code {
            // 0nnn - SYS addr - ignored by modern interpreters

            // 00EE - RET
            0x00ee => { // Return from a subroutine
                self.pc = self.stack[self.sp as usize];
                self.sp -= 1;
            }
            
            // 00E0 - CLS
            0x00e0 => { // Clear the display
                self.display.clear();
            }
            
            // NOP
            0x0000 => (), // Do nothing
            _ => return Err(Chip8Error::UnrecognizedOpcode(op_code.code, self.pc - 2)),
        }
        Ok(())
    }

    // 1nnn - JP addr
    fn execute_1nnn( &mut self, op_code: OpCode) { // Jump to location nnn
        let addr = op_code.addr();
        self.pc = addr;
    }

    // 2nnn - CALL addr
    fn execute_2nnn( &mut self, op_code: OpCode) { // Call subroutine at nnn
        self.sp += 1;
        self.stack[self.sp as usize] = self.pc;
        let addr = op_code.addr();
        self.pc = addr;
    }

    // 3xkk - SE Vx, byte
    fn execute_3xkk( &mut self, op_code: OpCode) { // Skip next instruction if Vx = kk
        let vx = op_code.vx();
        let data = op_code.byte();
        if self.v[vx] == data {
            self.pc += 2;
        }
    }

    // 4xkk - SNE Vx, byte
    fn execute_4xkk( &mut self, op_code: OpCode) { // Skip next instruction if Vx != kk
        let vx = op_code.vx();
        let data = op_code.byte();
        if self.v[vx] != data {
            self.pc += 2;
        }
    }

    // 5xy0 - SE Vx, Vy
    fn execute_5xy0( &mut self, op_code: OpCode) -> Result<(), Chip8Error>{ // Skip next instruction if Vx = Vy
        // Check if last nibble is 0, if not, it's an invalid opcode
        if op_code.nibble() != 0x0 { 
            return Err(Chip8Error::UnrecognizedOpcode(op_code.code, self.pc - 2));
        }

        let vx = op_code.vx(); 
        let vy = op_code.vy();
        if self.v[vx] == self.v[vy] {
            self.pc += 2;
        }
        Ok(())
    }

    // 6xkk - LD Vx, byte
    fn execute_6xkk( &mut self, op_code: OpCode) { // Set Vx = kk
        let vx = op_code.vx();
        let data = op_code.byte();
        self.v[vx] = data;
    }

    // 7xkk - ADD Vx, byte
    fn execute_7xkk( &mut self, op_code: OpCode) { // Set Vx = Vx + kk
        let vx = op_code.vx();
        let data = op_code.byte();
        self.v[vx] = self.v[vx].wrapping_add(data);
    }

    // Starts with 8 - Arithmetic operations
    fn execute_8nnn( &mut self, op_code: OpCode) -> Result<(), Chip8Error> {
        let vx = op_code.vx();
        let vy = op_code.vy();
        match op_code.nibble() {
            
            // 8xy0 - LD Vx, Vy
            0x0 => { // Set Vx = Vy
                self.v[vx] = self.v[vy];
            }
            
            // 8xy1 - OR Vx, Vy
            0x1 => { // Set Vx = Vx OR Vy
                self.v[vx] |= self.v[vy];
            }
            
            // 8xy2 - AND Vx, Vy
            0x2 => { // Set Vx = Vx AND Vy
                self.v[vx] &= self.v[vy];
            } 
            
            // 8xy3 - XOR Vx, Vy
            0x3 => { // Set Vx = Vx XOR Vy
                self.v[vx] ^= self.v[vy];
            }
            
            // 8xy4 - ADD Vx, Vy
            0x4 => { // Set Vx = Vx + Vy, set VF = carry
                let (sum, carry) = self.v[vx].overflowing_add(self.v[vy]);
                self.v[FLAG_REGISTER] = carry as u8;
                self.v[vx] = sum;
            }

            // 8xy5 - SUB Vx, Vy
            0x5 => { // Set Vx = Vx - Vy, set VF = NOT borrow
                let (diff, borrow) = self.v[vx].overflowing_sub(self.v[vy]);
                self.v[FLAG_REGISTER] = (!borrow) as u8;
                self.v[vx] = diff;
            }

            // 8xy6 - SHR Vx {, Vy}
            0x6 => { // Set Vx = Vx SHR 1, set VF = LSb of Vx
                self.v[FLAG_REGISTER] = self.v[vx] & 1;
                self.v[vx] >>= 1;
            }
            
            // 8xy7 - SUBN Vx, Vy
            0x7 => { // Set Vx = Vy - Vx, set VF = NOT borrow
                let (diff, borrow) = self.v[vy].overflowing_sub(self.v[vx]);
                self.v[FLAG_REGISTER] = (!borrow) as u8;
                self.v[vx] = diff;
            }

            // 8xyE - SHL Vx {, Vy}
            0xe => { // Set Vx = Vx SHL 1, set VF = MSB of Vx
                self.v[FLAG_REGISTER] = self.v[vx] >> 7;
                self.v[vx] <<= 1;
            }
            _ => return Err(Chip8Error::UnrecognizedOpcode(op_code.code, self.pc - 2)),
        }
        Ok(())
    }

    // 9xy0 SNE Vx, Vy
    fn execute_9xy0( &mut self, op_code: OpCode) -> Result<(), Chip8Error> { // Skip next instruction if Vx != Vy
        // Check if last nibble is 0, if not, it's an invalid opcode
        if op_code.nibble() != 0x0 { 
            return Err(Chip8Error::UnrecognizedOpcode(op_code.code, self.pc - 2));
        }
 
        let vx = op_code.vx();
        let vy = op_code.vy();
        if self.v[vx] != self.v[vy] {
            self.pc += 2;
        }
        Ok(())
    }

     // Annn - LD I, addr
    fn execute_annn( &mut self, op_code: OpCode) { // Set I = nnn
        let addr = op_code.addr();
        self.idx = addr;
    }

    // Bnnn - JP V0, addr
    fn execute_bnnn( &mut self, op_code: OpCode) { // Jump to location nnn + V0
        let addr = op_code.addr();
        self.pc = addr + self.v[0] as u16;
    }

    // Cxkk - RND Vx, byte
    fn execute_cxkk( &mut self, op_code: OpCode) { // Set Vx = random byte AND kk
        let vx = op_code.vx();
        let data = op_code.byte();
        let rnd: u8 = rand::random();
        self.v[vx] = data & rnd;
    }

    // Dxyn - DRW Vx, Vy, nibble
    fn execute_dxyn(&mut self, op_code: OpCode, mem: &Memory) { // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision
        let vx = op_code.vx();
        let vy = op_code.vy();
        let height = op_code.nibble() as usize;
        
        // Read sprite from memory
        let sprite = (0..height)
            .map(|offset| mem.read_byte(self.idx + offset as u16));
    
        let x = self.v[vx] as usize;
        let y = self.v[vy] as usize;
        
        // Draw sprite and set collision flag
        self.v[FLAG_REGISTER] = self.display.draw(x, y, sprite) as u8; 
    }

    // Ennn - Keyboard operations
    fn execute_ennn( &mut self, op_code: OpCode) -> Result<(), Chip8Error> { 
        let vx = op_code.vx();
        if let Some(key) = self.keyboard.get_by_value(self.v[vx]) {
            match op_code.byte() {

                // Ex9E - SKP Vx
                0x9e => { // Skip next instruction if key with the value of Vx is pressed
                    if self.display.is_key_down(*key) {
                        self.pc += 2;
                    }
                },

                // ExA1 - SKNP Vx
                0xa1 => { // Skip next instruction if key with the value of Vx is not pressed
                    if !self.display.is_key_down(*key) {
                        self.pc += 2;
                    }
                },
                _ => return Err(Chip8Error::UnrecognizedOpcode(op_code.code, self.pc - 2)),
            }
        }
        Ok(())
    }

    // Fnnn - Miscellaneous operations
    fn execute_fnnn( &mut self, op_code: OpCode, mem: &mut Memory) -> Result<(), Chip8Error> { // Starts with F
        let vx = op_code.vx();
        match op_code.byte() {

            // Fx07 - LD Vx, DT
            0x07 => { // Set Vx = delay timer value
                self.v[vx] = self.dt;
            }
            
            // Fx0A - LD Vx, K
            0x0a => {  // Wait for a key press, store the value of the key in Vx
                // Loop that will continue until a key press is detected
                loop {
                    self.display.update()?; // Update display
            
                    // Check if a key is pressed
                    if let Some(key) = self.display.get_key_press(&self.keyboard) {
                        self.v[vx] = key;
                        return Ok(());
                    }
            
                    // Sleep to reduce CPU usage while waiting for key press
                    thread::sleep(Duration::from_millis(MS_DELAY));
            
                    // Handle window closing during wait
                    if !self.display.is_open() {
                        return Ok(());
                    }
                }
            }

            // Fx15 - LD DT, Vx
            0x15 => { // Set delay timer = Vx
                self.dt = self.v[vx];
            }
            
            // Fx18 - LD ST, Vx
            0x18 => { // Set sound timer = Vx
                self.st = self.v[vx];
            }

            // Fx1E - ADD I, Vx
            0x1e => { // Set I = I + Vx
                self.idx += self.v[vx] as u16;
            }

            // Fx29 - LD F, Vx
            0x29 => { // Set I = location of sprite for digit Vx
                self.idx = self.v[vx] as u16 * SPRITE_SIZE; // Each sprite is 5 bytes long from 0x00 to 0x4F
            }

            // Fx33 - LD B, Vx
            0x33 => { // Store BCD representation of Vx in memory locations I, I+1, I+2
                mem.write_byte(self.idx, self.v[vx] / 100);
                mem.write_byte(self.idx + 1, (self.v[vx] % 100) / 10);
                mem.write_byte(self.idx + 2, self.v[vx] % 10);
            }

            // Fx55 - LD [I], Vx
            0x55 => { // Store registers V0 through Vx in memory starting at location I
                for i in 0..=vx {
                    mem.write_byte(self.idx + i as u16, self.v[i]);
                }
                self.idx += vx as u16 + 1;
            }

            // Fx65 - LD Vx, [I]
            0x65 => { // Read registers V0 through Vx from memory starting at location I
                for i in 0..=vx {
                    self.v[i] = mem.read_byte(self.idx + i as u16);
                }
                self.idx += vx as u16 + 1;
            }
            _ => return Err(Chip8Error::UnrecognizedOpcode(op_code.code, self.pc - 2)),
        }
        Ok(())
    }

    pub fn set_colors(&mut self, filled: u32, empty: u32) {
        self.display.set_colors(filled, empty);
    }

    pub fn with_bindings(&mut self, bindings: HashMap<u8, Key>) {
        self.keyboard = Keys::from(bindings);
    }

    pub fn insert_binding(&mut self, key: u8, value: Key) {
        self.keyboard.insert(key, value);
    }

    pub fn set_scale(&mut self, scale: Scale) {
        self.display.set_scale(scale);
    }
}

struct OpCode {
    code: u16,
}

impl OpCode {
    fn new(code: u16) -> Self { OpCode { code }}
    fn vx (&self) -> usize { ((self.code >> 8) & 0x000f) as usize }
    fn vy (&self) -> usize { ((self.code >> 4) & 0x000f) as usize }
    fn nibble (&self) -> u8 { (self.code & 0x000f) as u8 }
    fn byte (&self) -> u8 { (self.code & 0x00ff) as u8 }
    fn addr (&self) -> u16 { self.code & 0x0fff }
}