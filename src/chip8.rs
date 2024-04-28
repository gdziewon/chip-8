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
        pc: 0x200, // set PC to 0x200
        sp: 0x00,
        stack: [0x0000; STACK_DEPTH],
        display: [[false; DISPLAY_HEIGHT]; DISPLAY_WIDTH]
    }
}

fn execute( &mut self, op_code: u16, mem: &mut Memory, window: &mut Window) {
    match op_code >> 12 { // match by most significant 4bits

        0x0 => match op_code { // Starts with 0

            0x00ee => { // 00EE - RET
                self.pc = self.stack[self.sp as usize];
                self.sp -= 1;
            }

            0x00e0 => { // 00E0 - CLS
                self.display = [[false; DISPLAY_HEIGHT]; DISPLAY_WIDTH];
            }
            _ => () // NOP
        }

        0x1 => { // 1nnn - JP addr
            let addr = Chip8::get_addr(op_code);
            self.pc = addr;
            return;
        }

        0x2 => { // 2nnn - CALL addr
            self.sp += 1;
            self.stack[self.sp as usize] = self.pc;
            let addr = Chip8::get_addr(op_code);
            self.pc = addr;
            return;
        }

        0x3 => { // 3xkk - SE Vx, byte
            let vx = Chip8::get_vx(op_code);
            let data = Chip8::get_data_byte(op_code);
            if self.v[vx] == data {
                self.pc += 2;
            }
        }

        0x4 => { // 4xkk - SNE Vx, byte
            let vx = Chip8::get_vx(op_code);
            let data = Chip8::get_data_byte(op_code);
            if self.v[vx] != data {
                self.pc += 2;
            }
        }

        0x5 => { 
            if op_code & 0xf == 0 { // 5xy0 - SE Vx, Vy
                let vx = Chip8::get_vx(op_code);
                let vy = Chip8::get_vy(op_code);
                if self.v[vx] == self.v[vy] {
                    self.pc += 2;
                }
            }
        }

        0x6 => { // 6xkk - LD Vx, byte
            let vx = Chip8::get_vx(op_code);
            let data = Chip8::get_data_byte(op_code);
            self.v[vx] = data;
        }

        0x7 => { // 7xkk - ADD Vx, byte
            let vx = Chip8::get_vx(op_code);
            let data = Chip8::get_data_byte(op_code);
            self.v[vx] = self.v[vx].wrapping_add(data);
        }

        0x8 => { // Starts with 8
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
        }

        0x9 => {
            if op_code & 0xf == 0 { // 9xy0 SNE Vx, Vy
                let vx = Chip8::get_vx(op_code);
                let vy = Chip8::get_vy(op_code);
                if self.v[vx] != self.v[vy] {
                    self.pc += 2;
                }
            }
        }

        0xa => { // Annn - LD I, addr
            let addr = Chip8::get_addr(op_code);
            self.idx = addr;
        }

        0xb => { // Bnnn - JP V0, addr
            let addr = Chip8::get_addr(op_code);
            self.pc = addr + self.v[0] as u16;
            return;
        }

        0xc => { // Cxkk - RND Vx, byte
            let vx = Chip8::get_vx(op_code);
            let data = Chip8::get_data_byte(op_code);
            let rnd: u8 = rand::thread_rng().gen();
            self.v[vx] = data & rnd;
        }

        0xd => { // Dxyn - DRW Vx, Vy, nibble
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
        }

        0xe => { // Starts with E
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
        }

        0xf => { // Starts with F
            let vx = Chip8::get_vx(op_code);
            match op_code & 0x00ff {
                0x07 => { // Fx07 - LD Vx, DT
                    self.v[vx] = self.dt;
                }
                0x0a => { // Fx0A - LD Vx, K
                    let mut key_pressed = false;
                    while !key_pressed {
                        if let Some(key) = window.get_keys_pressed(KeyRepeat::No).get(0) {
                            if let Some(chip8_key) = KEYS.get_by_right(key) {
                                self.v[vx] = *chip8_key;
                                key_pressed = true;
                            }
                        }
                        window.update();
                        std::thread::sleep(std::time::Duration::from_millis(5));
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
                    mem.write_byte(self.idx, self.v[vx] / 100);
                    mem.write_byte(self.idx + 1, (self.v[vx] % 100) / 10);
                    mem.write_byte(self.idx + 2, self.v[vx] % 10);
                }
                0x55 => { // Fx55 - LD [I], Vx
                    for i in 0..vx {
                        mem.write_byte(self.idx, self.v[i]);
                        self.idx += 1;
                    }
                    self.idx += 1;
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
        }
        _ => ()
    }
    self.pc += 2;
}

pub fn run( &mut self, mem: &mut Memory ) {

    let mut display_ctl = display::Display::new();

    while display_ctl.window.is_open() {
        display_ctl.update(&self.display);

        let instruction: u16 = mem.get_2bytes(self.pc);
        if instruction == 0x0fff { // HALT
            return;
        }
        
        self.execute(instruction, mem, &mut display_ctl.window);

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