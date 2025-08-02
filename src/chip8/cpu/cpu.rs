use std::sync::atomic::AtomicU8;
use std::thread;
use std::time::Duration;
use std::sync::{Arc, atomic::Ordering};

use crate::errors::Chip8Error;
use crate::chip8::io::IO;
use super::opcode::{Addr, Nib};
use super::timer_clock::TimerClock;
use super::registers::Registers;

use super::opcode::OpCode;
use crate::chip8::memory::Memory;

pub const PROGRAM_START: u16 = 0x200;
const STACK_DEPTH: usize = 16;
const MS_DELAY: u64 = 1;
const SPRITE_SIZE: u16 = 5;

pub struct CPU {
    // Registers
    v: Registers, // 16 general purpose 8-bit registers
    idx: Addr, // 12-bit address register

    // Timers - counts down at 60hz to 0
    dt: Arc<AtomicU8>, // delay timer
    st: Arc<AtomicU8>, // sound timer

    pc: Addr, // Program counter
    sp: u8, // Stack pointer
    stack: [Addr; STACK_DEPTH], // 16 16-bit stack fields

    _timer_clock: TimerClock
}

impl CPU {
    pub fn new() -> Self {
        let dt = Arc::new(AtomicU8::new(0));
        let st = Arc::new(AtomicU8::new(0));
        let mut timer_clock = TimerClock::new(dt.clone(), st.clone());
        timer_clock.start();

        CPU {
            v: Registers::new(), // todo: this can be its own struct, for Nib indexing
            idx: Addr::new(),
            dt,
            st,
            pc: Addr::from(PROGRAM_START),
            sp: 0x00,
            stack: [Addr::new(); STACK_DEPTH],
            _timer_clock: timer_clock
        }
    }

    pub fn shutdown(&mut self) { // should be called only on started
        self._timer_clock.shutdown();
    }

    pub fn execute(&mut self, mem: &mut Memory, io: &mut IO) -> Result<(), Chip8Error> { // todo: handle st and dt decrementation, maybe seperate thread?
        let instruction = mem.get_instruction(self.pc);
        let opcode = OpCode::decode(instruction);
        self.pc+=2;

        match opcode {
            OpCode::NoOp => (),
            OpCode::ClearScreen => self.cleared_screen(io),
            OpCode::Return => self.return_subroutine(),
            OpCode::Jump(addr) => self.jump_addr(addr),
            OpCode::Call(addr) => self.call_addr(addr),
            OpCode::SkipEqualByte(x, byte) => self.skip_eq_byte(x, byte),
            OpCode::SkipNotEqualByte(x, byte) => self.skip_neq_byte(x, byte),
            OpCode::SkipEqualReg(x, y) => self.skip_eq_reg(x, y),
            OpCode::LoadByte(x, byte) => self.load_byte(x, byte),
            OpCode::AddByte(x, byte) => self.add_byte(x, byte),
            OpCode::LoadReg(x, y) => self.load_reg(x, y),
            OpCode::OrReg(x, y) => self.or_reg(x, y),
            OpCode::AndReg(x, y) => self.and_reg(x, y),
            OpCode::XorReg(x, y) => self.xor_reg(x, y),
            OpCode::AddReg(x, y) => self.add_reg(x, y),
            OpCode::SubReg(x, y) => self.sub_reg(x, y),
            OpCode::ShiftRight(x, _) => self.shr_reg(x),
            OpCode::SubNot(x, y) => self.subn_reg(x, y),
            OpCode::ShiftLeft(x, _) => self.shl_reg(x),
            OpCode::SkipNotEqualReg(x, y) => self.skip_neq_reg(x, y),
            OpCode::LoadIndex(addr) => self.load_idx(addr),
            OpCode::JumpV0(addr) => self.jump_v0(addr),
            OpCode::RandomByte(x, byte) => self.random_byte(x, byte),
            OpCode::Draw(x, y, n) => self.draw(x, y, n, mem, io),
            OpCode::SkipKeyPressed(x) => self.skip_key_pressed(x, io),
            OpCode::SkipKeyNotPressed(x) => self.skip_key_not_pressed(x, io),
            OpCode::LoadDelay(x) => self.load_delay(x),
            OpCode::WaitKey(x) => self.wait_key(x, io),
            OpCode::SetDelay(x) => self.set_delay(x),
            OpCode::SetSound(x) => self.set_sound(x),
            OpCode::AddToIndex(x) => self.add_idx(x),
            OpCode::LoadFont(x) => self.load_sprite(x),
            OpCode::LoadBCD(x) => self.load_bcd(x, mem),
            OpCode::StoreRegs(x) => self.store_regs(x,  mem),
            OpCode::LoadRegs(x) => self.load_regs(x, mem),
        }

        Ok(())
    }

    pub fn sound_timer(&self) -> u8 {
        self.st.load(Ordering::Relaxed)
    }

    fn return_subroutine(&mut self) {
        self.pc = self.stack[self.sp as usize];
        self.sp -= 1; // todo -> is this ok?
    }

    fn cleared_screen(&mut self, io: &mut IO) {
        io.display_clear();
    }

    fn jump_addr(&mut self, addr: Addr) {
        self.pc = addr;
    }

    fn call_addr(&mut self, addr: Addr) {
        self.sp += 1;
        self.stack[self.sp as usize] = self.pc;
        self.pc = addr;
    }

    fn skip_eq_byte(&mut self, vx: Nib, byte: u8) {
        if self.v[vx] == byte {
            self.pc += 2;
        }
    }

    fn skip_neq_byte(&mut self, vx: Nib, byte: u8) {
        if self.v[vx] != byte {
            self.pc += 2;
        }
    }

    fn skip_eq_reg(&mut self, vx: Nib, vy: Nib) { // todo: last nibble should be 0 for this one, check decoding
        if self.v[vx] == self.v[vy] {
            self.pc += 2;
        }
    }

    fn load_byte(&mut self, vx: Nib, byte: u8) {
        self.v[vx] = byte;
    }

    fn add_byte(&mut self, vx: Nib, byte: u8) {
        self.v[vx] = self.v[vx].wrapping_add(byte);
    }

    fn load_reg(&mut self, vx: Nib, vy: Nib) {
        self.v[vx] = self.v[vy];
    }

    fn or_reg(&mut self, vx: Nib, vy: Nib) {
        self.v[vx] |= self.v[vy];
    }

    fn and_reg(&mut self, vx: Nib, vy: Nib) {
        self.v[vx] &= self.v[vy];
    }

    fn xor_reg(&mut self, vx: Nib, vy: Nib) {
        self.v[vx] ^= self.v[vy];
    }

    fn add_reg(&mut self, vx: Nib, vy: Nib) {
        let (sum, carry) = self.v[vx].overflowing_add(self.v[vy]);
        self.v.set_flag(carry as u8);
        self.v[vx] = sum;
    }

    fn sub_reg(&mut self, vx: Nib, vy: Nib) {
        let (diff, borrow) = self.v[vx].overflowing_sub(self.v[vy]);
        self.v.set_flag((!borrow) as u8);
        self.v[vx] = diff;
    }

    fn shr_reg(&mut self, vx: Nib) { // todo! in enum we have vy here also, not needed
        let underflow = self.v[vx] & 1;
        self.v.set_flag(underflow);
        self.v[vx] >>= 1;
    }

    fn subn_reg(&mut self, vx: Nib, vy: Nib) {
        let (diff, borrow) = self.v[vy].overflowing_sub(self.v[vx]);
        self.v.set_flag((!borrow) as u8);
        self.v[vx] = diff;
}

    fn shl_reg(&mut self, vx: Nib) { // todo! in enum we have vy here also, not needed
        let overflow = self.v[vx] >> 7;
        self.v.set_flag(overflow);
        self.v[vx] <<= 1;
    }

    fn skip_neq_reg(&mut self, vx: Nib, vy: Nib) { // todo! last nibble needs to be 0, check decode
        if self.v[vx] != self.v[vy] {
            self.pc += 2;
        }
    }

    fn load_idx(&mut self, addr: Addr) {
        self.idx = addr;
    }

    fn jump_v0(&mut self, addr: Addr) {
        self.pc = addr + self.v.v0().into();
    }

    fn random_byte(&mut self, vx: Nib, byte: u8) {
        let rnd: u8 = rand::random();
        self.v[vx] = byte & rnd;
    }

    fn draw(&mut self, vx: Nib, vy: Nib, height: Nib, mem: &mut Memory, io: &mut IO) {
        // Read sprite from memory
        let sprite = (0..height.value())
            .map(|offset| mem.read_byte(self.idx + offset as u16));

        let x = self.v[vx] as usize;
        let y = self.v[vy] as usize;

        // Draw sprite and set collision flag
        let collision = io.display_draw(x, y, sprite);
        self.v.set_flag(collision as u8); // todo: some methods for flag register?
    }

    // Ennn - Keyboard operations
    fn skip_key_pressed(&mut self, vx: Nib, io: &IO) {
        if io.is_key_down(self.v[vx]) {
            self.pc += 2;
        }
    }

    fn skip_key_not_pressed(&mut self, vx: Nib, io: &IO) {
        if !io.is_key_down(self.v[vx]) {
            self.pc += 2;
        }
    }

    fn load_delay(&mut self, vx: Nib) {
        self.v[vx] = self.dt.load(Ordering::Relaxed);
    }

    fn wait_key(&mut self, vx: Nib, io: &mut IO) { // todo: gotta refactor that
        loop {
            let _ = io.display_update(); // todo: might return error btw

            // Check if a key is pressed
            if let Some(key) = io.get_key_press() {
                self.v[vx] = key;
                break;
            }

            // Sleep to reduce CPU usage while waiting for key press
            thread::sleep(Duration::from_millis(MS_DELAY));

            // Handle window closing during wait
            if !io.display_is_open() {
                break;
            }
        }
    }

    fn set_delay(&mut self, vx: Nib) {
        self.dt.store(self.v[vx], Ordering::Relaxed);
    }

    fn set_sound(&mut self, vx: Nib) {
        self.st.store(self.v[vx], Ordering::Relaxed);
    }

    fn add_idx(&mut self, vx: Nib) {
        self.idx += self.v[vx] as u16;
    }

    fn load_sprite(&mut self, vx: Nib) {
        self.idx = Addr::from( SPRITE_SIZE * self.v[vx] as u16); // Each sprite is 5 bytes long from 0x00 to 0x4F
    }

    fn load_bcd(&mut self, vx: Nib, mem: &mut Memory) {
        mem.write_byte(self.idx, self.v[vx] / 100);
        mem.write_byte(self.idx + 1, (self.v[vx] % 100) / 10);
        mem.write_byte(self.idx + 2, self.v[vx] % 10);
    }

    fn store_regs(&mut self, vx: Nib, mem: &mut Memory) {
        for i in 0..=vx.value() {
            let nib = Nib::from(i);
            mem.write_byte(self.idx + i as u16, self.v[nib]);
        }
        self.idx += vx.value() as u16 + 1;
    }

    fn load_regs(&mut self, vx: Nib, mem: &mut Memory) {
        for i in 0..=vx.value() {
            let nib = Nib::from(i);
            self.v[nib] = mem.read_byte(self.idx + i as u16);
        }
        self.idx += vx.value() as u16 + 1;
    }
}