mod display;
pub mod memory;
pub mod errors;

pub use memory::{Memory, MemoryEntry, MemoryAddress};
use errors::Chip8Error;
use display::Display;

use rand::Rng;
use minifb::{Key, KeyRepeat, Window};
use bimap::BiMap;
use lazy_static::lazy_static;
use rodio::{OutputStream, Sink, source::{SineWave, Source}};
use std::{cmp::Ordering, thread, time::Duration, sync::{Arc, Mutex}};

pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;
pub const SCALE: usize = 15;

pub const MEMORY_SIZE: usize = 1024 * 4;
pub const PROGRAM_START: u16 = 0x200;

const MS_DELAY: u64 = 100;

const NUM_REGISTERS: usize = 8;
const FLAG_REGISTER: usize = 7;
const STACK_DEPTH: usize = 16;

const SINEWAVE_FREQUENCY: f32 = 440.0;
const TIMERS_FREQUENCY: u64 = 1000 / 60;

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
dt: Arc<Mutex<u8>>, // delay timer
st: Arc<Mutex<u8>>, // sound timer

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
        dt: Arc::new(Mutex::new(0)),
        st: Arc::new(Mutex::new(0)),
        pc: PROGRAM_START,
        sp: 0x00,
        stack: [0x0000; STACK_DEPTH],
        display: [[false; DISPLAY_HEIGHT]; DISPLAY_WIDTH]
    }
}

fn execute( &mut self, op_code: u16, mem: &mut Memory, window: &mut Window) -> Result<(), Chip8Error> {
    let op_type = OpCodeType::from_u16(op_code)?;
    match op_type {
        OpCodeType::Zero => self.execute_0nnn(op_code)?,
        OpCodeType::One => self.execute_1nnn(op_code),
        OpCodeType::Two => self.execute_2nnn(op_code),
        OpCodeType::Three => self.execute_3xkk(op_code)?,
        OpCodeType::Four => self.execute_4xkk(op_code)?,
        OpCodeType::Five => self.execute_5xy0(op_code)?,
        OpCodeType::Six => self.execute_6xkk(op_code)?,
        OpCodeType::Seven => self.execute_7xkk(op_code)?,
        OpCodeType::Eight => self.execute_8nnn(op_code)?,
        OpCodeType::Nine => self.execute_9xy0(op_code)?,
        OpCodeType::A => self.execute_annn(op_code),
        OpCodeType::B => self.execute_bnnn(op_code),
        OpCodeType::C => self.execute_cxkk(op_code)?,
        OpCodeType::D => self.execute_dxyn(op_code, &mem)?,
        OpCodeType::E => self.execute_ennn(op_code, &window)?,
        OpCodeType::F => self.execute_fnnn(op_code, mem, window)?,
    }
    Ok(())
}

pub fn run( &mut self, mem: &mut Memory ) -> Result<(), Chip8Error> {
    self.run_timers();
    let mut display_ctl = Display::new()?;

    while display_ctl.window.is_open() {
        display_ctl.update(&self.display)?;

        let instruction: u16 = mem.get_instruction(self.pc)?;
        if instruction == 0x0fff { // HALT
            return Ok(());
        }

        self.execute(instruction, mem, &mut display_ctl.window)?;

        thread::sleep(Duration::from_millis(MS_DELAY)); // delay
    }
    Ok(())
}

fn run_timers(&self) {
    let dt = self.dt.clone();
    let st = self.st.clone();

    thread::spawn(move || {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        let source = SineWave::new(SINEWAVE_FREQUENCY).repeat_infinite();
        sink.append(source);
        sink.pause();

        loop {
            thread::sleep(Duration::from_millis(TIMERS_FREQUENCY));

            let mut sound_timer = st.lock().unwrap();
            let mut delay_timer = dt.lock().unwrap();

            if *sound_timer > 0 {
                sink.play();
                *sound_timer -= 1;
            } else {
                sink.pause();
            }

            if *delay_timer > 0 {
                *delay_timer -= 1;
            }
        }
    });
}

fn execute_0nnn( &mut self, op_code: u16) -> Result<(), Chip8Error>{
    match op_code {
        0x00ee => { // 00EE - RET
            self.pc = self.stack[self.sp as usize];
            self.sp -= 1;
        }

        0x00e0 => { // 00E0 - CLS
            self.display = [[false; DISPLAY_HEIGHT]; DISPLAY_WIDTH];
        }
        0x0000 => (), // NOP
        _ => return Err(Chip8Error::UnrecognizedOpcode(op_code)),
    }
    self.pc += 2;
    Ok(())
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

fn execute_3xkk( &mut self, op_code: u16) -> Result<(), Chip8Error> { // 3xkk - SE Vx, byte
    let vx = Chip8::get_vx(op_code)?;
    let data = Chip8::get_data_byte(op_code);

    self.pc += if self.v[vx] == data { 4 } else { 2 };
    Ok(())
}

fn execute_4xkk( &mut self, op_code: u16) -> Result<(), Chip8Error> { // 4xkk - SNE Vx, byte
    let vx = Chip8::get_vx(op_code)?;
    let data = Chip8::get_data_byte(op_code);

    self.pc += if self.v[vx] != data { 4 } else { 2 };
    Ok(())
}

fn execute_5xy0( &mut self, op_code: u16) -> Result<(), Chip8Error>{ 
    match op_code & 0xf { // 5xy0 - SE Vx, Vy
        0x0 => {
            let vx = Chip8::get_vx(op_code)?;
            let vy = Chip8::get_vy(op_code)?;
            self.pc += if self.v[vx] == self.v[vy] { 4 } else { 2 };
            Ok(())
        }
        _ => Err(Chip8Error::UnrecognizedOpcode(op_code)),
    }
}

fn execute_6xkk( &mut self, op_code: u16) -> Result<(), Chip8Error> { // 6xkk - LD Vx, byte
    let vx = Chip8::get_vx(op_code)?;
    let data = Chip8::get_data_byte(op_code);
    self.v[vx] = data;
    self.pc += 2;
    Ok(())
}

fn execute_7xkk( &mut self, op_code: u16) -> Result<(), Chip8Error> { // 7xkk - ADD Vx, byte
    let vx = Chip8::get_vx(op_code)?;
    let data = Chip8::get_data_byte(op_code);
    self.v[vx] = self.v[vx].wrapping_add(data);
    self.pc += 2;
    Ok(())
}

fn execute_8nnn( &mut self, op_code: u16) -> Result<(), Chip8Error> { // Start with 8
    let vx = Chip8::get_vx(op_code)?;
    let vy = Chip8::get_vy(op_code)?;
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
            self.v[FLAG_REGISTER] = if self.v[vx] as u16 + self.v[vy] as u16 > 0xff { 1 } else { 0 };
            self.v[vx] = self.v[vx].wrapping_add(self.v[vy]);
        }

        0x5 => { // 8xy5 - SUB Vx, Vy
            self.v[FLAG_REGISTER] = if self.v[vx] >= self.v[vy] { 1 } else { 0 };
            self.v[vx] = self.v[vx].wrapping_sub(self.v[vy]);
        }

        0x6 => { // 8xy6 - SHR Vx {, Vy}
            self.v[FLAG_REGISTER] = self.v[vx] & 1;
            self.v[vx] >>= 1;
        }

        0x7 => { // 8xy7 - SUBN Vx, Vy
            self.v[FLAG_REGISTER] = if self.v[vy] >= self.v[vx] { 1 } else { 0 };
            self.v[vx] = self.v[vy].wrapping_sub(self.v[vx]);
        }

        0xe => { // 8xyE - SHL Vx {, Vy}
            self.v[FLAG_REGISTER] = self.v[vx] >> 7;
            self.v[vx] <<= 1;
        }
        _ => return Err(Chip8Error::UnrecognizedOpcode(op_code)),
    }
    self.pc += 2;
    Ok(())
}

fn execute_9xy0( &mut self, op_code: u16) -> Result<(), Chip8Error> { 
    match op_code & 0xf { // 9xy0 SNE Vx, Vy
        0x0 => {
            let vx = Chip8::get_vx(op_code)?;
            let vy = Chip8::get_vy(op_code)?;
            self.pc += if self.v[vx] != self.v[vy] { 4 } else { 2 };
            Ok(())
        }
        _ => Err(Chip8Error::UnrecognizedOpcode(op_code)),
    }
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

fn execute_cxkk( &mut self, op_code: u16) -> Result<(), Chip8Error>{ // Cxkk - RND Vx, byte
    let vx = Chip8::get_vx(op_code)?;
    let data = Chip8::get_data_byte(op_code);
    let rnd: u8 = rand::thread_rng().gen();
    self.v[vx] = data & rnd;
    self.pc += 2;
    Ok(())
}

fn execute_dxyn( &mut self, op_code: u16, mem: &Memory) -> Result<(), Chip8Error> { // Dxyn - DRW Vx, Vy, nibble
    let vx = Chip8::get_vx(op_code)?;
    let vy = Chip8::get_vy(op_code)?;
    let height = op_code & 0xf;

    self.v[FLAG_REGISTER] = 0; // Reset collision register

    for offset in 0..height {
        let sprite_byte = mem.read_byte(MemoryAddress::new(self.idx + offset as u16)?);
        
        for bit in 0..8 {
            let pixel = (sprite_byte >> (7 - bit)) & 1 != 0;
            let x = (self.v[vx] as usize + bit) % DISPLAY_WIDTH;
            let y = (self.v[vy] as usize + offset as usize) % DISPLAY_HEIGHT;

            if self.display[x][y] & pixel {
                self.v[FLAG_REGISTER] = 1; // Set collision flag
            }

            // XOR pixel
            self.display[x][y] ^= pixel;
        }
    }
    self.idx += height;
    self.pc += 2;
    Ok(())
}

fn execute_ennn( &mut self, op_code: u16, window: &Window) -> Result<(), Chip8Error> { // Starts with E
    let vx = Chip8::get_vx(op_code)?;
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
            _ => return Err(Chip8Error::UnrecognizedOpcode(op_code)),
        }
    }
    self.pc += 2;
    Ok(())
}

fn execute_fnnn( &mut self, op_code: u16, mem: &mut Memory, window: &mut Window) -> Result<(), Chip8Error> { // Starts with F
    let vx = Chip8::get_vx(op_code)?;
    match op_code & 0x00ff {
        0x07 => { // Fx07 - LD Vx, DT
            let dt_value = *self.dt.lock().unwrap();
            self.v[vx] = dt_value;
        }
        
        0x0a => { // Fx0A - LD Vx, K
            // loop that will continue until a key press is detected
            let mut key_pressed = None;
            while key_pressed.is_none() {
                window.update();
        
                // Check if any key pressed matches a mapped key
                if let Some(key) = window.get_keys_pressed(KeyRepeat::No).iter().find_map(|&k| KEYS.get_by_right(&k)) {
                    self.v[vx] = *key;
                    key_pressed = Some(*key);
                }
        
                // Sleep to reduce CPU usage while waiting for key press
                thread::sleep(Duration::from_millis(5));
        
                // Handle window closing during wait
                if !window.is_open() {
                    return Ok(());
                }
            }
        }

        0x15 => { // Fx15 - LD DT, Vx
            let mut dt_guard = self.dt.lock().unwrap();
            *dt_guard = self.v[vx];
        }

        0x18 => { // Fx18 - LD ST, Vx
            let mut st_guard = self.st.lock().unwrap();
            *st_guard = self.v[vx];
        }

        0x1e => { // Fx1E - ADD I, Vx
            self.idx += self.v[vx] as u16;
        }

        0x29 => { // Fx29 - LD F, Vx
            self.idx = self.v[vx] as u16 * 5;
        }

        0x33 => { // Fx33 - LD B, Vx
            mem.write_entry(MemoryEntry::new(self.idx, self.v[vx] / 100)?);
            mem.write_entry(MemoryEntry::new(self.idx + 1, (self.v[vx] % 100) / 10)?);
            mem.write_entry(MemoryEntry::new(self.idx + 2, self.v[vx] % 10)?);
        }

        0x55 => { // Fx55 - LD [I], Vx
            for i in 0..vx {
                mem.write_entry(MemoryEntry::new(self.idx + i as u16, self.v[i])?);
            }
            self.idx += vx as u16 + 1;
        }

        0x65 => { // Fx65 - LD Vx, [I]
            for i in 0..vx {
                self.v[i] = mem.read_byte(MemoryAddress::new(self.idx + i as u16)?);
            }
            self.idx += vx as u16 + 1;
        }
        _ => return Err(Chip8Error::UnrecognizedOpcode(op_code)),
    }
    self.pc += 2;
    Ok(())
}

fn get_vx(op_code: u16) -> Result<usize, Chip8Error> {
    let vx = ((op_code >> 8) & 0x000f) as usize;
    Chip8::validate_register(vx)
}

fn get_vy(op_code: u16) -> Result<usize, Chip8Error> {
    let vy = ((op_code >> 4) & 0x000f) as usize;
    Chip8::validate_register(vy)
}

fn validate_register(register_idx: usize) -> Result<usize, Chip8Error> {
    // Prevents programs from using registers outside the range as well as the flag register
    match register_idx.cmp(&FLAG_REGISTER) {
        Ordering::Greater => Err(Chip8Error::RegisterIndexOutOfBounds(register_idx)),
        Ordering::Equal => Err(Chip8Error::FlagRegisterUsed),
        Ordering::Less => Ok(register_idx),
    }
}

fn get_data_byte(op_code: u16) -> u8 {
    (op_code & 0x00ff) as u8
}

fn get_addr(op_code: u16) -> u16 {
    op_code & 0x0fff
}
}

enum OpCodeType {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    A,
    B,
    C,
    D,
    E,
    F,
}

impl OpCodeType {
    fn from_u16(op_code: u16) -> Result<Self, Chip8Error> {
        match op_code >> 12 {
            0x0 => Ok(Self::Zero),
            0x1 => Ok(Self::One),
            0x2 => Ok(Self::Two),
            0x3 => Ok(Self::Three),
            0x4 => Ok(Self::Four),
            0x5 => Ok(Self::Five),
            0x6 => Ok(Self::Six),
            0x7 => Ok(Self::Seven),
            0x8 => Ok(Self::Eight),
            0x9 => Ok(Self::Nine),
            0xA => Ok(Self::A),
            0xB => Ok(Self::B),
            0xC => Ok(Self::C),
            0xD => Ok(Self::D),
            0xE => Ok(Self::E),
            0xF => Ok(Self::F),
            _ => Err(Chip8Error::UnrecognizedOpcode(op_code)), // Impossible to happend due to the bit shift
        }
    }
}
