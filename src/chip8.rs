mod display;
pub mod memory;

pub use memory::{Memory, MemoryEntry};

use rand::Rng;
use minifb::{Key, KeyRepeat, Window};
use bimap::BiMap;
use lazy_static::lazy_static;
use std::{thread, time::Duration};

pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;
pub const SCALE: usize = 15;

pub const MEMORY_SIZE: usize = 1024 * 4;
pub const PROGRAM_START: u16 = 0x200;

const MS_DELAY: u64 = 100;

const NUM_REGISTERS: usize = 8;
const STACK_DEPTH: usize = 16;

lazy_static! {
    // TODO: Change bindings
    static ref KEYS: BiMap<u8, Key> = { // Key bindings
        let mut keys = BiMap::new();
        keys.insert(0x0, Key::Key0);
        keys.insert(0x1, Key::Key1);
        keys.insert(0x2, Key::Key2);
        keys.insert(0x3, Key::Key3);
        keys.insert(0x4, Key::Key4);
        keys.insert(0x5, Key::Key5);
        keys.insert(0x6, Key::Key6);
        keys.insert(0x7, Key::Key7);
        keys.insert(0x8, Key::Key8);
        keys.insert(0x9, Key::Key9);
        keys.insert(0xa, Key::A);
        keys.insert(0xb, Key::B);
        keys.insert(0xc, Key::C);
        keys.insert(0xd, Key::D);
        keys.insert(0xe, Key::E);
        keys.insert(0xf, Key::F);
        keys.insert(0xff, Key::Escape);
        keys
    };
}

pub struct Chip8 {
// Registers
v: [u8; NUM_REGISTERS], // 8 general purpose 8-bit registers
idx: u16, // 16-bit address register

// Timers  - count down at 60hz to 0
dt: u8, // delay timer
st: u8, // sound timer

pc: u16, // program counter
sp: u8, // stack pointer
stack: [u16; STACK_DEPTH], // 16 16-bit stack fields

display: [[bool; DISPLAY_HEIGHT]; DISPLAY_WIDTH],
}


impl Chip8 {

pub fn new() -> Self {
    Chip8 {
        v: [0x00; NUM_REGISTERS],
        idx: 0x0000,
        dt: 0x00,
        st: 0x00,
        pc: PROGRAM_START, // set PC to 0x200
        sp: 0x00,
        stack: [0x0000; STACK_DEPTH],
        display: [[false; DISPLAY_HEIGHT]; DISPLAY_WIDTH]
    }
}

fn execute( &mut self, op_code: u16, mem: &mut Memory, window: &mut Window) {
    match op_code >> 12 {
        0x0 => self.execute_0nnn(op_code),
        0x1 => self.execute_1nnn(op_code),
        0x2 => self.execute_2nnn(op_code),
        0x3 => self.execute_3xkk(op_code),
        0x4 => self.execute_4xkk(op_code),
        0x5 => self.execute_5xy0(op_code),
        0x6 => self.execute_6xkk(op_code),
        0x7 => self.execute_7xkk(op_code),
        0x8 => self.execute_8nnn(op_code),
        0x9 => self.execute_9xy0(op_code),
        0xA => self.execute_annn(op_code),
        0xB => self.execute_bnnn(op_code),
        0xC => self.execute_cxkk(op_code),
        0xD => self.execute_dxyn(op_code, &mem),
        0xE => self.execute_ennn(op_code, &window),
        0xF => self.execute_fnnn(op_code, mem, window),
        _ => ()
    }
}

pub fn run( &mut self, mem: &mut Memory ) {

    let mut display_ctl = display::Display::new();

    while display_ctl.window.is_open() {
        display_ctl.update(&self.display);

        let instruction: u16 = mem.get_instruction(self.pc);
        if instruction == 0x0fff { // HALT
            return;
        }

        self.execute(instruction, mem, &mut display_ctl.window);

        thread::sleep(Duration::from_millis(MS_DELAY)); // delay
    }
}

fn execute_0nnn( &mut self, op_code: u16) {
    match op_code {
        0x00ee => { // 00EE - RET
            self.pc = self.stack[self.sp as usize];
            self.sp -= 1;
        }

        0x00e0 => { // 00E0 - CLS
            self.display = [[false; DISPLAY_HEIGHT]; DISPLAY_WIDTH];
        }
        _ => () // NOP
    }
    self.pc += 2;
}

fn execute_1nnn( &mut self, op_code: u16) { // 1nnn - JP addr
    let addr = Chip8::get_addr(op_code);
    self.pc = addr;
}

fn execute_2nnn( &mut self, op_code: u16) { // 2nnn - CALL addr 
    self.sp += 1;
    self.stack[self.sp as usize] = self.pc;
    let addr = Chip8::get_addr(op_code);
    self.pc = addr;
}

fn execute_3xkk( &mut self, op_code: u16) { // 3xkk - SE Vx, byte
    let vx = Chip8::get_vx(op_code);
    let data = Chip8::get_data_byte(op_code);
    if self.v[vx] == data {
        self.pc += 2;
    }
    self.pc += 2;
}

fn execute_4xkk( &mut self, op_code: u16) { // 4xkk - SNE Vx, byte
    let vx = Chip8::get_vx(op_code);
    let data = Chip8::get_data_byte(op_code);
    if self.v[vx] != data {
        self.pc += 2;
    }
    self.pc += 2;
}

fn execute_5xy0( &mut self, op_code: u16) { // 5xy0 - SE Vx, Vy
    if op_code & 0xf == 0 { 
        let vx = Chip8::get_vx(op_code);
        let vy = Chip8::get_vy(op_code);
        if self.v[vx] == self.v[vy] {
            self.pc += 2;
        }
    }
    self.pc += 2;
}

fn execute_6xkk( &mut self, op_code: u16) { // 6xkk - LD Vx, byte
    let vx = Chip8::get_vx(op_code);
    let data = Chip8::get_data_byte(op_code);
    self.v[vx] = data;
    self.pc += 2;
}

fn execute_7xkk( &mut self, op_code: u16) { // 7xkk - ADD Vx, byte
    let vx = Chip8::get_vx(op_code);
    let data = Chip8::get_data_byte(op_code);
    self.v[vx] = self.v[vx].wrapping_add(data);
    self.pc += 2;
}

fn execute_8nnn( &mut self, op_code: u16) { // Start with 8
    let vx = Chip8::get_vx(op_code);
    let vy = Chip8::get_vy(op_code);
    match op_code & 0xf {

        0x0 => { // 8xy0 - LD Vx, Vy
            self.v[vx] = self.v[vy];
        }

        0x1 => { // 8xy1 - OR Vx, Vy
            self.v[vx] |= self.v[vy];
        }

        0x2 => { // 8xy2 - AND Vx, Vy
            self.v[vx] &= self.v[vy];
        } 

        0x3 => { // 8xy3 - XOR Vx, Vy
            self.v[vx] ^= self.v[vy];
        }
        
        0x4 => { // 8xy4 - ADD Vx, Vy
            self.v[7] = if self.v[vx] as u16 + self.v[vy] as u16 > 0xff { 1 } else { 0 };
            self.v[vx] = self.v[vx].wrapping_add(self.v[vy]);
        }

        0x5 => { // 8xy5 - SUB Vx, Vy
            self.v[7] = if self.v[vx] >= self.v[vy] { 1 } else { 0 };
            self.v[vx] = self.v[vx].wrapping_sub(self.v[vy]);
        }

        0x6 => { // 8xy6 - SHR Vx {, Vy}
            self.v[7] = self.v[vx] & 1;
            self.v[vx] >>= 1;
        }

        0x7 => { // 8xy7 - SUBN Vx, Vy
            self.v[7] = if self.v[vy] >= self.v[vx] { 1 } else { 0 };
            self.v[vx] = self.v[vy].wrapping_sub(self.v[vx]);
        }

        0xe => { // 8xyE - SHL Vx {, Vy}
            self.v[7] = self.v[vx] >> 7;
            self.v[vx] <<= 1;
        }
        _ => () // NOP
    }
    self.pc += 2;
}

fn execute_9xy0( &mut self, op_code: u16) { // 9xy0 SNE Vx, Vy
    if op_code & 0xf == 0 { 
        let vx = Chip8::get_vx(op_code);
        let vy = Chip8::get_vy(op_code);
        if self.v[vx] != self.v[vy] {
            self.pc += 2;
        }
    }
    self.pc += 2;
}

fn execute_annn( &mut self, op_code: u16) { // Annn - LD I, addr
    let addr = Chip8::get_addr(op_code);
    self.idx = addr;
    self.pc += 2;
}

fn execute_bnnn( &mut self, op_code: u16) { // Bnnn - JP V0, addr
    let addr = Chip8::get_addr(op_code);
    self.pc = addr + self.v[0] as u16;
}

fn execute_cxkk( &mut self, op_code: u16) { // Cxkk - RND Vx, byte
    let vx = Chip8::get_vx(op_code);
    let data = Chip8::get_data_byte(op_code);
    let rnd: u8 = rand::thread_rng().gen();
    self.v[vx] = data & rnd;
    self.pc += 2;
}

fn execute_dxyn( &mut self, op_code: u16, mem: &Memory) { // Dxyn - DRW Vx, Vy, nibble
    let vx = Chip8::get_vx(op_code) as usize;
    let vy = Chip8::get_vy(op_code) as usize;
    let height = op_code & 0xF;

    self.v[7] = 0; // Reset collision register

    for offset in 0..height {
        let sprite_byte = mem.get_byte(self.idx + offset);
        for bit in 0..8 {
            let pixel = (sprite_byte >> (7 - bit)) & 1 != 0;
            let x = (self.v[vx] as usize + bit) % DISPLAY_WIDTH;
            let y = (self.v[vy] as usize + offset as usize) % DISPLAY_HEIGHT;

            if self.display[x][y] & pixel {
                self.v[7] = 1; // Set collision flag
            }

            // XOR pixel
            self.display[x][y] ^= pixel;
        }
    }
    self.idx += height;
    self.pc += 2;
}

fn execute_ennn( &mut self, op_code: u16, window: &Window) { // Starts with E
    let vx = Chip8::get_vx(op_code);
    if let Some(key) = KEYS.get_by_left(&self.v[vx]) {
        match op_code & 0x00ff {
            0x9e => { // Ex9E - SKP Vx
                if window.is_key_down(*key) {
                    self.pc += 2;
                }
            },
            0xa1 => { // ExA1 - SKNP Vx
                if !window.is_key_down(*key) {
                    self.pc += 2;
                }
            },
            _ => ()
        }
    }
    self.pc += 2;
}

fn execute_fnnn( &mut self, op_code: u16, mem: &mut Memory, window: &mut Window) { // Starts with F
    let vx = Chip8::get_vx(op_code);
    match op_code & 0x00ff {
        0x07 => { // Fx07 - LD Vx, DT
            self.v[vx] = self.dt;
        }

        0x0a => { // Fx0A - LD Vx, K
            let mut key_pressed = false;
            while !key_pressed {
                // Wait for key press
                if let Some(key) = window.get_keys_pressed(KeyRepeat::No).get(0) {
                    if let Some(chip8_key) = KEYS.get_by_right(key) {
                        self.v[vx] = *chip8_key;
                        key_pressed = true;
                    }
                }

                // Keep window responsive
                window.update();
                thread::sleep(Duration::from_millis(5));
            }
        }

        0x15 => { // Fx15 - LD DT, Vx
            self.dt = self.v[vx];
        }

        0x18 => { // Fx18 - LD ST, Vx
            self.st = self.v[vx];
        }

        0x1e => { // Fx1E - ADD I, Vx
            self.idx += self.v[vx] as u16;
        }

        0x29 => { // Fx29 - LD F, Vx
            self.idx = self.v[vx] as u16 * 5;
        }

        0x33 => { // Fx33 - LD B, Vx
            mem.write_entry(MemoryEntry::new(self.idx, self.v[vx] / 100).unwrap());
            mem.write_entry(MemoryEntry::new(self.idx + 1, (self.v[vx] % 100) / 10).unwrap());
            mem.write_entry(MemoryEntry::new(self.idx + 2, self.v[vx] % 10).unwrap());
        }

        0x55 => { // Fx55 - LD [I], Vx
            for i in 0..vx {
                mem.write_entry(MemoryEntry::new(self.idx + i as u16, self.v[i]).unwrap());
            }
            self.idx += vx as u16 + 1;
        }

        0x65 => { // Fx65 - LD Vx, [I]
            for i in 0..vx {
                self.v[i] = mem.get_byte(self.idx);
                self.idx += 1;
            }
            self.idx += 1;
        }
        _ => ()
    }
    self.pc += 2;
}

fn get_vx(op_code: u16) -> usize{
    ((op_code >> 8) & 0x000f) as usize
}

fn get_vy(op_code: u16) -> usize {
    ((op_code >> 4) & 0x000f) as usize
}

fn get_data_byte(op_code: u16) -> u8 {
    (op_code & 0x00ff) as u8
}

fn get_addr(op_code: u16) -> u16 {
    op_code & 0x0fff
}
}