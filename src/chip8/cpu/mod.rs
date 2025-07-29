mod opcode;

use std::thread;
use std::time::Duration;

use crate::errors::Chip8Error;
use crate::chip8::io::IO;
use opcode::{Addr, Nib};

use opcode::OpCode;
use crate::chip8::memory::Memory;

pub const PROGRAM_START: u16 = 0x200;
const STACK_DEPTH: usize = 16;
const NUM_REGISTERS: usize = 16;
const FLAG_REGISTER: usize = 0xF;
const MS_DELAY: u64 = 1;
const SPRITE_SIZE: u16 = 5;

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

impl CPU {
    pub(super) fn new() -> Self {
        CPU {
            v: [0x00; NUM_REGISTERS],
            idx: 0x0000,
            dt: 0,
            st: 0,
            pc: PROGRAM_START,
            sp: 0x00,
            stack: [0x0000; STACK_DEPTH]
        }
    }

    pub(super) fn execute(&mut self, mem: &mut Memory, io: &mut IO) -> Result<(), Chip8Error> { // todo: handle st and dt decrementation, maybe seperate thread?
        let instruction = mem.get_instruction(self.pc);
        let opcode = OpCode::decode(instruction);
        self.pc+=2;

        match opcode {
            OpCode::NoOp => (),
            OpCode::ClearScreen => self.cleared_display(io),
            OpCode::Return => self.return_subroutine(),
            OpCode::Jump(addr) => self.jump_addr(addr),
            OpCode::Call(addr) => self.call_addr(addr),
            OpCode::SkipEqualByte(x, byte) => self.skip_eq_reg_byte(x, byte),
            OpCode::SkipNotEqualByte(x, byte) => self.skip_neq_reg_byte(x, byte),
            OpCode::SkipEqualReg(x, y) => self.skip_reg_eq_reg(x, y),
            OpCode::LoadByte(x, byte) => self.load_reg_byte(x, byte),
            OpCode::AddByte(x, byte) => self.add_reg_byte(x, byte),
            OpCode::LoadReg(x, y) => self.load_reg_reg(x, y),
            OpCode::OrReg(x, y) => self.or_reg_reg(x, y),
            OpCode::AndReg(x, y) => self.and_reg_reg(x, y),
            OpCode::XorReg(x, y) => self.xor_reg_reg(x, y),
            OpCode::AddReg(x, y) => self.add_reg_reg(x, y),
            OpCode::SubReg(x, y) => self.sub_reg_reg(x, y),
            OpCode::ShiftRight(x, _) => self.shr_reg_reg(x),
            OpCode::SubNot(x, y) => self.subn_reg_reg(x, y),
            OpCode::ShiftLeft(x, _) => self.shl_reg_reg(x),
            OpCode::SkipNotEqualReg(x, y) => self.skip_neq_reg_reg(x, y),
            OpCode::LoadIndex(addr) => self.load_idx_addr(addr),
            OpCode::JumpV0(addr) => self.jump_v0_addr(addr),
            OpCode::RandomByte(x, byte) => self.random_reg_byte(x, byte),
            OpCode::Draw(x, y, n) => self.draw(x, y, n, mem, io),
            OpCode::SkipKeyPressed(x) => self.skip_key_pressed(x, io),
            OpCode::SkipKeyNotPressed(x) => self.skip_key_not_pressed(x, io),
            OpCode::LoadDelay(x) => self.load_reg_dt(x),
            OpCode::WaitKey(x) => self.load_reg_key(x, io),
            OpCode::SetDelay(x) => self.load_dt_reg(x),
            OpCode::SetSound(x) => self.load_st_reg(x),
            OpCode::AddToIndex(x) => self.add_idx_reg(x),
            OpCode::LoadFont(x) => self.load_idx_sprite(x),
            OpCode::LoadBCD(x) => self.load_bcd_vx(x, mem),
            OpCode::StoreRegs(x) => self.load_idx_regs(x,  mem),
            OpCode::LoadRegs(x) => self.load_regs_idx(x, mem),
        }

        Ok(())
    }

    pub fn sound_timer(&self) -> u8 {
        self.st
    }

    fn return_subroutine(&mut self) {
        self.pc = self.stack[self.sp as usize];
        self.sp -= 1; // todo -> is this ok?
    }

    fn cleared_display(&mut self, io: &mut IO) {
        io.display_clear();
    }

    fn jump_addr(&mut self, addr: Addr) {
        self.pc = addr.addr();
    }

    fn call_addr(&mut self, addr: Addr) {
        self.sp += 1;
        self.stack[self.sp as usize] = self.pc;
        self.pc = addr.addr();
    }

    fn skip_eq_reg_byte(&mut self, vx: Nib, byte: u8) {
        if self.v[vx.idx()] == byte {
            self.pc += 2;
        }
    }

    fn skip_neq_reg_byte(&mut self, vx: Nib, byte: u8) {
        if self.v[vx.idx()] != byte {
            self.pc += 2;
        }
    }

    fn skip_reg_eq_reg(&mut self, vx: Nib, vy: Nib) { // todo: last nibble should be 0 for this one, check decoding
        if self.v[vx.idx()] == self.v[vy.idx()] {
            self.pc += 2;
        }
    }

    fn load_reg_byte(&mut self, vx: Nib, byte: u8) {
        self.v[vx.idx()] = byte;
    }

    fn add_reg_byte(&mut self, vx: Nib, byte: u8) {
        self.v[vx.idx()] = self.v[vx.idx()].wrapping_add(byte);
    }

    fn load_reg_reg(&mut self, vx: Nib, vy: Nib) {
        self.v[vx.idx()] = self.v[vy.idx()];
    }

    fn or_reg_reg(&mut self, vx: Nib, vy: Nib) {
        self.v[vx.idx()] |= self.v[vy.idx()];
    }

    fn and_reg_reg(&mut self, vx: Nib, vy: Nib) {
        self.v[vx.idx()] &= self.v[vy.idx()];
    }

    fn xor_reg_reg(&mut self, vx: Nib, vy: Nib) {
        self.v[vx.idx()] ^= self.v[vy.idx()];
    }

    fn add_reg_reg(&mut self, vx: Nib, vy: Nib) {
        let (sum, carry) = self.v[vx.idx()].overflowing_add(self.v[vy.idx()]);
        self.v[FLAG_REGISTER] = carry as u8;
        self.v[vx.idx()] = sum;
    }

    fn sub_reg_reg(&mut self, vx: Nib, vy: Nib) {
        let (diff, borrow) = self.v[vx.idx()].overflowing_sub(self.v[vy.idx()]);
        self.v[FLAG_REGISTER] = (!borrow) as u8;
        self.v[vx.idx()] = diff;
    }

    fn shr_reg_reg(&mut self, vx: Nib) { // todo! in enum we have vy here also, not needed
        self.v[FLAG_REGISTER] = self.v[vx.idx()] & 1;
        self.v[vx.idx()] >>= 1;
    }

    fn subn_reg_reg(&mut self, vx: Nib, vy: Nib) {
        let (diff, borrow) = self.v[vy.idx()].overflowing_sub(self.v[vx.idx()]);
        self.v[FLAG_REGISTER] = (!borrow) as u8;
        self.v[vx.idx()] = diff;
}

    fn shl_reg_reg(&mut self, vx: Nib) { // todo! in enum we have vy here also, not needed
        self.v[FLAG_REGISTER] = self.v[vx.idx()] >> 7;
        self.v[vx.idx()] <<= 1;
    }

    fn skip_neq_reg_reg(&mut self, vx: Nib, vy: Nib) { // todo! last nibble needs to be 0, check decode
        if self.v[vx.idx()] != self.v[vy.idx()] {
            self.pc += 2;
        }
    }

    fn load_idx_addr(&mut self, addr: Addr) {
        self.idx = addr.addr();
    }

    fn jump_v0_addr(&mut self, addr: Addr) {
        self.pc = addr.addr() + self.v[0] as u16;
    }

    fn random_reg_byte(&mut self, vx: Nib, byte: u8) {
        let rnd: u8 = rand::random();
        self.v[vx.idx()] = byte & rnd;
    }

    fn draw(&mut self, vx: Nib, vy: Nib, height: Nib, mem: &mut Memory, io: &mut IO) {
        // Read sprite from memory
        let sprite = (0..height.idx())
            .map(|offset| mem.read_byte(self.idx + offset as u16));

        let x = self.v[vx.idx()] as usize;
        let y = self.v[vy.idx()] as usize;

        // Draw sprite and set collision flag
        self.v[FLAG_REGISTER] = io.display_draw(x, y, sprite) as u8;
    }

    // Ennn - Keyboard operations
    fn skip_key_pressed(&mut self, vx: Nib, io: &IO) {
        if io.is_key_down(self.v[vx.idx()]) {
            self.pc += 2;
        }
    }

    fn skip_key_not_pressed(&mut self, vx: Nib, io: &IO) {
        if !io.is_key_down(self.v[vx.idx()]) {
            self.pc += 2;
        }
    }

    fn load_reg_dt(&mut self, vx: Nib) {
        self.v[vx.idx()] = self.dt;
    }

    fn load_reg_key(&mut self, vx: Nib, io: &mut IO) { // todo: gotta refactor that
        loop {
            let _ = io.display_update(); // todo: might return error btw

            // Check if a key is pressed
            if let Some(key) = io.get_key_press() {
                self.v[vx.idx()] = key;
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

    fn load_dt_reg(&mut self, vx: Nib) {
        self.dt = self.v[vx.idx()];
    }

    fn load_st_reg(&mut self, vx: Nib) {
        self.st = self.v[vx.idx()];
    }

    fn add_idx_reg(&mut self, vx: Nib) {
        self.idx += self.v[vx.idx()] as u16;
    }

    fn load_idx_sprite(&mut self, vx: Nib) {
        self.idx = self.v[vx.idx()] as u16 * SPRITE_SIZE; // Each sprite is 5 bytes long from 0x00 to 0x4F
    }

    fn load_bcd_vx(&mut self, vx: Nib, mem: &mut Memory) {
        mem.write_byte(self.idx, self.v[vx.idx()] / 100);
        mem.write_byte(self.idx + 1, (self.v[vx.idx()] % 100) / 10);
        mem.write_byte(self.idx + 2, self.v[vx.idx()] % 10);
    }

    fn load_idx_regs(&mut self, vx: Nib, mem: &mut Memory) {
        for i in 0..=vx.idx() {
            mem.write_byte(self.idx + i as u16, self.v[i]);
        }
        self.idx += vx.nib() as u16 + 1;
    }

    fn load_regs_idx(&mut self, vx: Nib, mem: &mut Memory) {
        for i in 0..=vx.idx() {
            self.v[i] = mem.read_byte(self.idx + i as u16);
        }
        self.idx += vx.nib() as u16 + 1;
    }
}