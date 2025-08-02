use std::ops::{Add, AddAssign};
use crate::errors::Chip8Error;

const ADDR_MASK: u16 = 0x0FFF;
const NIB_MASK: u8 = 0x0F;

#[derive(Clone, Copy)]
pub struct Addr(u16);

impl Addr {
    pub fn new() -> Self {
        Self::from(0)
    }

    pub fn from(val: u16) -> Self { // todo: should I return result here?
        Self(val & ADDR_MASK)
    }

    pub fn value(&self) -> u16 {
        self.0
    }
}

impl Add<u16> for Addr {
    type Output = Addr;

    fn add(self, rhs: u16) -> Self::Output {
        Self::from(self.0.wrapping_add(rhs as u16))
    }
}

impl AddAssign<u16> for Addr {
    fn add_assign(&mut self, rhs: u16) {
        self.0 = self.0 + rhs
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Nib(u8);

impl Nib {
    pub const fn from(val: u8) -> Self { // todo: should I return result here?
        Self(val & NIB_MASK)
    }

    pub fn value(&self) -> u8 {
        self.0
    }
}

pub enum OpCode {
    NoOp,                     // 0000 - NOP
    ClearScreen,              // 00E0 - CLS
    Return,                   // 00EE - RET
    Jump(Addr),               // 1aaa - JP addr
    Call(Addr),               // 2aaa - CALL addr
    SkipEqualByte(Nib, u8),   // 3xkk - SE Vx, byte
    SkipNotEqualByte(Nib, u8),// 4xkk - SNE Vx, byte
    SkipEqualReg(Nib, Nib),   // 5xy0 - SE Vx, Vy
    LoadByte(Nib, u8),        // 6xkk - LD Vx, byte
    AddByte(Nib, u8),         // 7xkk - ADD Vx, byte
    LoadReg(Nib, Nib),        // 8xy0 - LD Vx, Vy
    OrReg(Nib, Nib),          // 8xy1 - OR Vx, Vy
    AndReg(Nib, Nib),         // 8xy2 - AND Vx, Vy
    XorReg(Nib, Nib),         // 8xy3 - XOR Vx, Vy
    AddReg(Nib, Nib),         // 8xy4 - ADD Vx, Vy
    SubReg(Nib, Nib),         // 8xy5 - SUB Vx, Vy
    ShiftRight(Nib, Nib),     // 8xy6 - SHR Vx {, Vy}
    SubNot(Nib, Nib),         // 8xy7 - SUBN Vx, Vy
    ShiftLeft(Nib, Nib),      // 8xyE - SHL Vx {, Vy}
    SkipNotEqualReg(Nib, Nib),// 9xy0 - SNE Vx, Vy
    LoadIndex(Addr),          // Aaaa - LD I, addr
    JumpV0(Addr),             // Baaa - JP V0, addr
    RandomByte(Nib, u8),      // Cxkk - RND Vx, byte
    Draw(Nib, Nib, Nib),      // Dxyn - DRW Vx, Vy, nibble
    SkipKeyPressed(Nib),      // Ex9E - SKP Vx
    SkipKeyNotPressed(Nib),   // ExA1 - SKNP Vx
    LoadDelay(Nib),           // Fx07 - LD Vx, DT
    WaitKey(Nib),             // Fx0A - LD Vx, K
    SetDelay(Nib),            // Fx15 - LD DT, Vx
    SetSound(Nib),            // Fx18 - LD ST, Vx
    AddToIndex(Nib),          // Fx1E - ADD I, Vx
    LoadFont(Nib),            // Fx29 - LD F, Vx
    LoadBCD(Nib),             // Fx33 - LD B, Vx
    StoreRegs(Nib),           // Fx55 - LD [I], Vx
    LoadRegs(Nib),            // Fx65 - LD Vx, [I]
}

struct Deconstructed {
    code: u16,
    group: Nib,
    addr: Addr,
    x: Nib,
    y: Nib,
    n: Nib,
    kk: u8
}

/*
    Each opcode is represented as a 16-bit value with the structure:

        15 14 13 12 11 10 9 8 7 6 5 4 3 2 1 0
        [    G    ] [          addr         ]
                    [   x   ] [  y  ] [  n  ]
                              [     kk      ]
    where:
        G - 4-bit instruction group
        addr - 12-bit address
        x, y - 4-bit register index
        n - 4-bit numeric value
        kk - 8-bit immediate value
*/
impl Deconstructed {
    fn new(code: u16) -> Self {
        let group = Nib::from((code >> 12) as u8);
        let addr = Addr::from(code & 0xFFF);
        let x = Nib::from((code >> 8) as u8);
        let y = Nib::from((code >> 4) as u8);
        let n = Nib::from(code as u8);
        let kk = (code & 0x00ff) as u8;
        Self {
            code,
            group,
            addr,
            x,
            y,
            n,
            kk
        }
    }
}

impl OpCode {
    pub(super) fn decode(code: u16) -> Result<Self, Chip8Error> {
        use OpCode::*;

        let dec = Deconstructed::new(code);

        match dec.group.value() {
            0x0 => match dec.addr.value() {
                0x0E0 => Ok(ClearScreen),
                0x0EE => Ok(Return),
                0x000 => Ok(NoOp),
                _ => Err(Chip8Error::UnrecognizedOpcode(dec.code)),
            },

            0x1 => Ok(Jump(dec.addr)),

            0x2 => Ok(Call(dec.addr)),

            0x3 => Ok(SkipEqualByte(dec.x, dec.kk)),

            0x4 => Ok(SkipNotEqualByte(dec.x, dec.kk)),

            0x5 if dec.n.value() == 0x0 => Ok(SkipEqualReg(dec.x, dec.y)),

            0x6 => Ok(LoadByte(dec.x, dec.kk)),

            0x7 => Ok(AddByte(dec.x, dec.kk)),

            0x8 => match dec.n.value() {
                0x0 => Ok(LoadReg(dec.x, dec.y)),
                0x1 => Ok(OrReg(dec.x, dec.y)),
                0x2 => Ok(AndReg(dec.x, dec.y)),
                0x3 => Ok(XorReg(dec.x, dec.y)),
                0x4 => Ok(AddReg(dec.x, dec.y)),
                0x5 => Ok(SubReg(dec.x, dec.y)),
                0x6 => Ok(ShiftRight(dec.x, dec.y)),
                0x7 => Ok(SubNot(dec.x, dec.y)),
                0xE => Ok(ShiftLeft(dec.x, dec.y)),
                _ => Err(Chip8Error::UnrecognizedOpcode(dec.code)),
            },

            0x9 if dec.n.value() == 0x0 => Ok(SkipNotEqualReg(dec.x, dec.y)),

            0xA => Ok(LoadIndex(dec.addr)),

            0xB => Ok(JumpV0(dec.addr)),

            0xC => Ok(RandomByte(dec.x, dec.kk)),

            0xD => Ok(Draw(dec.x, dec.y, dec.n)),

            0xE => match dec.kk {
                0x9E => Ok(SkipKeyPressed(dec.x)),
                0xA1 => Ok(SkipKeyNotPressed(dec.x)),
                _ => Err(Chip8Error::UnrecognizedOpcode(dec.code)),
            },

            0xF => match dec.kk {
                0x07 => Ok(LoadDelay(dec.x)),
                0x0A => Ok(WaitKey(dec.x)),
                0x15 => Ok(SetDelay(dec.x)),
                0x18 => Ok(SetSound(dec.x)),
                0x1E => Ok(AddToIndex(dec.x)),
                0x29 => Ok(LoadFont(dec.x)),
                0x33 => Ok(LoadBCD(dec.x)),
                0x55 => Ok(StoreRegs(dec.x)),
                0x65 => Ok(LoadRegs(dec.x)),
                _ => Err(Chip8Error::UnrecognizedOpcode(dec.code)),
            },

            _ => Err(Chip8Error::UnrecognizedOpcode(dec.code)),
        }
    }
}