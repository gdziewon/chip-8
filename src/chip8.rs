mod display;
mod keys;
pub mod memory;
pub mod errors;
pub mod opcode;

#[cfg(test)]
mod tests;

pub use memory::Memory;
use errors::Chip8Error;
use display::Display;
use keys::Keys;
use opcode::{Addr, Nib, OpCode};

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

pub struct CPU {
    // Registers
    v: [u8; NUM_REGISTERS], // 16 general purpose 8-bit registers
    idx: u16, // 16-bit address register

    // Timers - counts down at 60hz to 0
    dt: u8, // delay timer
    st: u8, // sound timer

    pc: u16, // Program counter
    sp: u8, // Stack pointer
    stack: [u16; STACK_DEPTH], // 16 16-bit stack fields
}

pub struct Chip8 {
    cpu: CPU,

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

        let cpu = CPU{ v: [0x00; NUM_REGISTERS],
            idx: 0x0000,
            dt: 0,
            st: 0,
            pc: PROGRAM_START,
            sp: 0x00,
            stack: [0x0000; STACK_DEPTH]};

        Chip8 {
            cpu,
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
            let instruction_code: u16 = mem.get_instruction(self.cpu.pc);
            let opcode = OpCode::decode(instruction_code);

            // Increment program counter
            self.cpu.pc += 2;

            // Execute instruction
            self.execute(opcode, mem)?;

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
        if self.cpu.st > 0 { // Decrement sound timer at 60hz
            self.audio.play(); // Play sound when sound timer is greater than 0
            self.cpu.st -= 1;
        } else {
            self.audio.pause(); // Pause sound when sound timer is 0
        }

        // if self.cpu.st > 0 { // Decrement delay timer at 60hz
        //     self.cpu.st -= 1;
        // }
    }

    // Executes given opcode dividing them by their first nibble
    fn execute( &mut self, opcode: OpCode, mem: &mut Memory) -> Result<(), Chip8Error> {
        match opcode {
            OpCode::NOP => (),
            OpCode::CLS => self.cleared_display(),
            OpCode::RET => self.return_subroutine(),
            OpCode::JPa(addr) => self.jump_addr(addr),
            OpCode::CALLa(addr) => self.call_addr(addr),
            OpCode::SEvb(x, byte) => self.skip_eq_reg_byte(x, byte),
            OpCode::SNEvb(x, byte) => self.skip_neq_reg_byte(x, byte),
            OpCode::SEvv(x, y) => self.skip_reg_eq_reg(x, y),
            OpCode::LDvb(x, byte) => self.load_reg_byte(x, byte),
            OpCode::ADDvb(x, byte) => self.add_reg_byte(x, byte),
            OpCode::LDvv(x, y) => self.load_reg_reg(x, y),
            OpCode::ORvv(x, y) => self.or_reg_reg(x, y),
            OpCode::ANDvv(x, y) => self.and_reg_reg(x, y),
            OpCode::XORvv(x, y) => self.xor_reg_reg(x, y),
            OpCode::ADDvv(x, y) => self.add_reg_reg(x, y),
            OpCode::SUBvv(x, y) => self.sub_reg_reg(x, y),
            OpCode::SHRvv(x, _) => self.shr_reg_reg(x),
            OpCode::SUBNvv(x, y) => self.subn_reg_reg(x, y),
            OpCode::SHLvv(x, _) => self.shl_reg_reg(x),
            OpCode::SNEvv(x, y) => self.skip_neq_reg_reg(x, y),
            OpCode::LDa(addr) => self.load_idx_addr(addr),
            OpCode::JPva(addr) => self.jump_v0_addr(addr),
            OpCode::RNDvb(x, byte) => self.random_reg_byte(x, byte),
            OpCode::DRWvvn(x, y, n) => self.draw(x, y, n, mem),
            OpCode::SKPv(x) => self.skip_key_pressed(x),
            OpCode::SKNPv(x) => self.skip_key_not_pressed(x),
            OpCode::LDvd(x) => self.load_reg_dt(x),
            OpCode::LDvk(x) => self.load_reg_key(x),
            OpCode::LDdv(x) => self.load_dt_reg(x),
            OpCode::LDsv(x) => self.load_st_reg(x),
            OpCode::ADDi(x) => self.add_idx_reg(x),
            OpCode::LDfv(x) => self.load_idx_sprite(x),
            OpCode::LDbv(x) => self.load_bcd_vx(x, mem),
            OpCode::LDiv(x) => self.load_idx_regs(x,  mem),
            OpCode::LDvi(x) => self.load_regs_idx(x, mem),
        }
        Ok(())
    }

    fn return_subroutine(&mut self) {
        self.cpu.pc = self.cpu.stack[self.cpu.sp as usize];
        self.cpu.sp -= 1; // todo -> is this ok?
    }

    fn cleared_display(&mut self) {
        self.display.clear();
    }

    fn jump_addr(&mut self, addr: Addr) {
        self.cpu.pc = addr.addr();
    }

    fn call_addr(&mut self, addr: Addr) {
        self.cpu.sp += 1;
        self.cpu.stack[self.cpu.sp as usize] = self.cpu.pc;
        self.cpu.pc = addr.addr();
    }

    fn skip_eq_reg_byte(&mut self, vx: Nib, byte: u8) {
        if self.cpu.v[vx.idx()] == byte {
            self.cpu.pc += 2;
        }
    }

    fn skip_neq_reg_byte(&mut self, vx: Nib, byte: u8) {
        if self.cpu.v[vx.idx()] != byte {
            self.cpu.pc += 2;
        }
    }

    fn skip_reg_eq_reg(&mut self, vx: Nib, vy: Nib) { // todo: last nibble should be 0 for this one, check decoding
        if self.cpu.v[vx.idx()] == self.cpu.v[vy.idx()] {
            self.cpu.pc += 2;
        }
    }

    fn load_reg_byte(&mut self, vx: Nib, byte: u8) {
        self.cpu.v[vx.idx()] = byte;
    }

    fn add_reg_byte(&mut self, vx: Nib, byte: u8) {
        self.cpu.v[vx.idx()] = self.cpu.v[vx.idx()].wrapping_add(byte);
    }

    fn load_reg_reg(&mut self, vx: Nib, vy: Nib) {
        self.cpu.v[vx.idx()] = self.cpu.v[vy.idx()];
    }

    fn or_reg_reg(&mut self, vx: Nib, vy: Nib) {
        self.cpu.v[vx.idx()] |= self.cpu.v[vy.idx()];
    }

    fn and_reg_reg(&mut self, vx: Nib, vy: Nib) {
        self.cpu.v[vx.idx()] &= self.cpu.v[vy.idx()];
    }

    fn xor_reg_reg(&mut self, vx: Nib, vy: Nib) {
        self.cpu.v[vx.idx()] ^= self.cpu.v[vy.idx()];
    }

    fn add_reg_reg(&mut self, vx: Nib, vy: Nib) {
        let (sum, carry) = self.cpu.v[vx.idx()].overflowing_add(self.cpu.v[vy.idx()]);
        self.cpu.v[FLAG_REGISTER] = carry as u8;
        self.cpu.v[vx.idx()] = sum;
    }

    fn sub_reg_reg(&mut self, vx: Nib, vy: Nib) {
        let (diff, borrow) = self.cpu.v[vx.idx()].overflowing_sub(self.cpu.v[vy.idx()]);
        self.cpu.v[FLAG_REGISTER] = (!borrow) as u8;
        self.cpu.v[vx.idx()] = diff;
    }

    fn shr_reg_reg(&mut self, vx: Nib) { // todo! in enum we have vy here also, not needed
        self.cpu.v[FLAG_REGISTER] = self.cpu.v[vx.idx()] & 1;
        self.cpu.v[vx.idx()] >>= 1;
    }

    fn subn_reg_reg(&mut self, vx: Nib, vy: Nib) {
        let (diff, borrow) = self.cpu.v[vy.idx()].overflowing_sub(self.cpu.v[vx.idx()]);
        self.cpu.v[FLAG_REGISTER] = (!borrow) as u8;
        self.cpu.v[vx.idx()] = diff;
}

    fn shl_reg_reg(&mut self, vx: Nib) { // todo! in enum we have vy here also, not needed
        self.cpu.v[FLAG_REGISTER] = self.cpu.v[vx.idx()] >> 7;
        self.cpu.v[vx.idx()] <<= 1;
    }

    fn skip_neq_reg_reg(&mut self, vx: Nib, vy: Nib) { // todo! last nibble needs to be 0, check decode
        if self.cpu.v[vx.idx()] != self.cpu.v[vy.idx()] {
            self.cpu.pc += 2;
        }
    }

    fn load_idx_addr(&mut self, addr: Addr) {
        self.cpu.idx = addr.addr();
    }

    fn jump_v0_addr(&mut self, addr: Addr) {
        self.cpu.pc = addr.addr() + self.cpu.v[0] as u16;
    }

    fn random_reg_byte(&mut self, vx: Nib, byte: u8) {
        let rnd: u8 = rand::random();
        self.cpu.v[vx.idx()] = byte & rnd;
    }

    fn draw(&mut self, vx: Nib, vy: Nib, height: Nib, mem: &Memory) {
        // Read sprite from memory
        let sprite = (0..height.idx())
            .map(|offset| mem.read_byte(self.cpu.idx + offset as u16));

        let x = self.cpu.v[vx.idx()] as usize;
        let y = self.cpu.v[vy.idx()] as usize;

        // Draw sprite and set collision flag
        self.cpu.v[FLAG_REGISTER] = self.display.draw(x, y, sprite) as u8;
    }

    // Ennn - Keyboard operations
    fn skip_key_pressed(&mut self, vx: Nib) {
        if let Some(key) = self.keyboard.get_by_value(self.cpu.v[vx.idx()]) {
            if self.display.is_key_down(*key) {
                self.cpu.pc += 2;
            }
        }
    }

    fn skip_key_not_pressed(&mut self, vx: Nib) {
        if let Some(key) = self.keyboard.get_by_value(self.cpu.v[vx.idx()]) {
            if !self.display.is_key_down(*key) {
                self.cpu.pc += 2;
            }
        }
    }

    fn load_reg_dt(&mut self, vx: Nib) {
        self.cpu.v[vx.idx()] = self.cpu.dt;
    }

    fn load_reg_key(&mut self, vx: Nib) { // todo: gotta refactor that
        loop {
            let _ = self.display.update(); // todo: might return error btw

            // Check if a key is pressed
            if let Some(key) = self.display.get_key_press(&self.keyboard) {
                self.cpu.v[vx.idx()] = key;
                break;
            }

            // Sleep to reduce CPU usage while waiting for key press
            thread::sleep(Duration::from_millis(MS_DELAY));

            // Handle window closing during wait
            if !self.display.is_open() {
                break;
            }
        }
    }

    fn load_dt_reg(&mut self, vx: Nib) {
        self.cpu.dt = self.cpu.v[vx.idx()];
    }

    fn load_st_reg(&mut self, vx: Nib) {
        self.cpu.st = self.cpu.v[vx.idx()];
    }

    fn add_idx_reg(&mut self, vx: Nib) {
        self.cpu.idx += self.cpu.v[vx.idx()] as u16;
    }

    fn load_idx_sprite(&mut self, vx: Nib) {
        self.cpu.idx = self.cpu.v[vx.idx()] as u16 * SPRITE_SIZE; // Each sprite is 5 bytes long from 0x00 to 0x4F
    }

    fn load_bcd_vx(&mut self, vx: Nib, mem: &mut Memory) {
        mem.write_byte(self.cpu.idx, self.cpu.v[vx.idx()] / 100);
        mem.write_byte(self.cpu.idx + 1, (self.cpu.v[vx.idx()] % 100) / 10);
        mem.write_byte(self.cpu.idx + 2, self.cpu.v[vx.idx()] % 10);
    }

    fn load_idx_regs(&mut self, vx: Nib, mem: &mut Memory) {
        for i in 0..=vx.idx() {
            mem.write_byte(self.cpu.idx + i as u16, self.cpu.v[i]);
        }
        self.cpu.idx += vx.nib() as u16 + 1;
    }

    fn load_regs_idx(&mut self, vx: Nib, mem: &mut Memory) {
        for i in 0..=vx.idx() {
            self.cpu.v[i] = mem.read_byte(self.cpu.idx + i as u16);
        }
        self.cpu.idx += vx.nib() as u16 + 1;
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

