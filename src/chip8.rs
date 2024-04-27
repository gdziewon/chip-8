mod display;
pub mod memory;

use rand::Rng;
use minifb::{Key, KeyRepeat, Window};
use bimap::BiMap;
use lazy_static::lazy_static;

use self::memory::Memory;

pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;
pub const SCALE: usize = 15;

const MS_DELAY: u64 = 300;

const NUM_REGISTERS: usize = 8;
const STACK_DEPTH: usize = 16;

lazy_static! {
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
        pc: 0x200,
        sp: 0x00,
        stack: [0x0000; STACK_DEPTH],
        display: [[false; DISPLAY_HEIGHT]; DISPLAY_WIDTH]
    }
}

fn execute( &mut self, op_code: u16, mem: &mut [u8], window: &Window) {
    match op_code >> 12 { // match by most significant 4bits
        0x0 => match op_code {
            0x00ee => { // RET
                self.pc = self.stack[self.sp as usize];
                self.sp -= 1;
            }
            0x0e0 => { // CLS
                self.display = [[false; DISPLAY_HEIGHT]; DISPLAY_WIDTH];
            }
            0x0000  => { // NOP
            } 
            _ => { // CALL (0nnn)
                self.sp += 1;
                self.stack[self.sp as usize] = self.pc;
                let addr = Chip8::get_addr(op_code);
                self.pc = addr;
            }
        }
        0x1 => { // JP addr
            let addr = Chip8::get_addr(op_code);
            self.pc = addr;
        }
        0x2 => { // CALL addr
            self.sp += 1;
            self.stack[self.sp as usize] = self.pc;
            let addr = Chip8::get_addr(op_code);
            self.pc = addr;
        }
        0x3 => { // SE Vx, byte
            let vx = Chip8::get_vx(op_code);
            let data = Chip8::get_data_byte(op_code);
            if self.v[vx] == data {
                self.idx += 2;
            }
        }
        0x4 => { // SNE Vx, byte
            let vx = Chip8::get_vx(op_code);
            let data = Chip8::get_data_byte(op_code);
            if self.v[vx] != data {
                self.pc += 2;
            }
        }
        0x5 => { // SE Vx, Vy
            let vx = Chip8::get_vx(op_code);
            let vy = Chip8::get_vy(op_code);
            if self.v[vx] == self.v[vy] {
                self.pc += 2;
            }
        }
        0x6 => { // LD Vx, byte
            let vx = Chip8::get_vx(op_code);
            let data = Chip8::get_data_byte(op_code);
            self.v[vx] = data;
        }
        0x7 => { // ADD Vx, byte
            let vx = Chip8::get_vx(op_code);
            let data = Chip8::get_data_byte(op_code);
            self.v[vx] += data;
        }
        0x8 => {
            let vx = Chip8::get_vx(op_code);
            let vy = Chip8::get_vy(op_code);
            match op_code & 0x000f {
                0x0 => { // LD Vx, Vy
                    self.v[vx] = self.v[vy];
                },
                0x1 => { // OR Vx, Vy
                    self.v[vx] = self.v[vx] | self.v[vy];
                },
                0x2 => { // AND Vx, Vy
                    self.v[vx] = self.v[vx] & self.v[vy];
                },  
                0x3 => { // XOR Vx, Vy
                    self.v[vx] = self.v[vx] ^ self.v[vy];
                }
                0x4 => { // ADD Vx, Vy
                    self.v[7] = if self.v[vx] as u16 + self.v[vy] as u16 > 0xff { 1 } else { 0 };
                    self.v[vx] = self.v[vx].wrapping_add(self.v[vy]);
                }
                0x5 => { // SUB Vx, Vy
                    self.v[7] = if self.v[vx] >= self.v[vy] { 1 } else { 0 };
                    self.v[vx] = self.v[vx].wrapping_sub(self.v[vy]);
                }
                0x6 => { // SHR Vx {, Vy}
                    self.v[7] = self.v[vx] & 1;
                    self.v[vx] >>= 1;
                }
                0x7 => { // SUBN Vx, Vy
                    self.v[7] = if self.v[vx] <= self.v[vy] { 1 } else { 0 };
                    self.v[vx] = self.v[vy].wrapping_sub(self.v[vx]);
                }
                0xe => { // SHL Vx {, Vy}
                    self.v[7] = self.v[vx] >> 7;
                    self.v[vx] <<= 1;
                }
                _ => ()
            }
        }
        0x9 => {
            let vx = Chip8::get_vx(op_code);
            let vy = Chip8::get_vy(op_code);
            if self.v[vx] != self.v[vy] {
                self.pc += 2;
            }
        }
        0xa => { // LD I, addr
            let addr = Chip8::get_addr(op_code);
            self.idx = addr;
        }
        0xb => { // JP V0, addr
            let addr = Chip8::get_addr(op_code);
            self.pc = addr + self.v[0] as u16;
            println!("Jumped to: {}", self.pc);
            return;
        }
        0xc => { // RND Vx, byte
            let vx = Chip8::get_vx(op_code);
            let data = Chip8::get_data_byte(op_code);
            let rnd : u8 = rand::thread_rng().gen();
            self.v[vx] = data & rnd;
        }
        0xd => { // DRW Vx, Vy, nibble
            let vx = Chip8::get_vx(op_code);
            let vy = Chip8::get_vy(op_code);
            let n = op_code & 0xf;
            for i in 0..n {
                let byte = mem[self.idx as usize];
                for j in 0..8 {
                    self.v[7] |= self.display[((self.v[vx] % DISPLAY_HEIGHT as u8) + j) as usize]
                    [((self.v[vy] % DISPLAY_WIDTH as u8) + i as u8) as usize] as u8
                    & ((byte >> (7 - j)) & 1); // set collision

                    self.display[((self.v[vx] % DISPLAY_HEIGHT as u8) + j) as usize]
                    [((self.v[vy] % DISPLAY_WIDTH as u8) + i as u8) as usize]
                    ^= ((byte >> (7 - j)) & 1) != 0; // xor sprites
                }
                self.idx += 1;
            }
        }
        0xe => {
            let vx = Chip8::get_vx(op_code);
            let key = KEYS.get_by_left(&self.v[vx]).unwrap_or(&Key::Escape);
            match op_code & 0x00ff {
            0x9e => { // SKP Vx
                if window.is_key_down(*key) {
                    self.pc += 2;
                }
            }
            0xa1 => { // SKNP Vx
                if !window.is_key_down(*key) {
                    self.pc += 2;
                }
            }
            _ => ()
        }}
        0xf => {
            let vx = Chip8::get_vx(op_code);
            match op_code & 0x00ff {
                0x07 => { // LD Vx, DT
                    self.v[vx] = self.dt;
                }
                0x0a => { // LD Vx, K
                    let mut key: u8 = 0xff;
                    while key == 0xff {
                        key = *KEYS.get_by_right(window.get_keys_pressed(KeyRepeat::No).get(0).unwrap_or(&Key::Escape)).unwrap();
                    }
                    self.v[vx] = key;
                }
                0x15 => { // LD DT, Vx
                    self.dt = self.v[vx];
                }
                0x18 => { // LD ST, Vx
                    self.st = self.v[vx];
                }
                0x1e => { // ADD I, Vx
                    self.idx += self.v[vx] as u16;
                }
                0x29 => { // LD F, Vx
                    // display
                }
                0x33 => { // LD B, Vx

                }
                0x55 => { // LD [I], Vx
                    for i in 0..vx {
                        mem[self.idx as usize] = self.v[i];
                        self.idx += 1;
                    }
                    self.idx += 1;
                }
                0x65 => {
                    for i in 0..vx {
                        self.v[i] = mem[self.idx as usize];
                        self.idx += 1;
                    }
                    self.idx += 1;
                }
                _ => ()
            }
        }
        _ => ()
    }
    self.pc += 2;
}

pub fn run( &mut self, mem: &mut Memory ) {

    let mut display_ctl = display::Display::new();

    while self.pc < 1024 * 4 {
        println!("PC: {}", self.pc);
        println!("V3: {}", self.v[3]);
        println!("V1: {}", self.v[1]);
        let instruction: u16 = (mem.memory[self.pc as usize] as u16) << 8 |
            (mem.memory[(self.pc + 1) as usize] as u16);
        if instruction == 0x0fff { // HALT
            return;
        }
        self.execute(instruction, &mut mem.memory, &display_ctl.window);
        
        display_ctl.update(&self.display);

        std::thread::sleep(std::time::Duration::from_millis(MS_DELAY)); // delay
    }
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