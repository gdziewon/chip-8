
use modular_bitfield::prelude::*;

#[bitfield(bits = 12)]
pub struct Addr {
    pub addr: B12
}

impl Addr {
    fn from(val: u16) -> Addr {
        Addr::new().with_addr(val)
    }
}

#[bitfield(bits = 4)]
pub struct Nib {
    pub nib: B4
}

impl Nib {
    fn from(val: u8) -> Nib {
        Nib::new().with_nib(val)
    }

    pub fn idx(&self) -> usize {
        self.nib() as usize
    }
}

pub enum OpCode {
    NoOp,                 // NOP
    ClearScreen,                 // 00E0 - CLS
    Return,                 // 00EE - RET
    Jump(Addr),           // 1nnn - JP addr
    Call(Addr),         // 2nnn - CALL addr
    SkipEqualByte(Nib, u8),       // 3xkk - SE Vx, byte
    SkipNotEqualByte(Nib, u8),      // 4xkk - SNE Vx, byte
    SkipEqualReg(Nib, Nib),      // 5xy0 - SE Vx, Vy
    LoadByte(Nib, u8),      // 6xkk - LD Vx, byte
    AddByte(Nib, u8),     // 7xkk - ADD Vx, byte
    LoadReg(Nib, Nib),      // 8xy0 - LD Vx, Vy
    OrReg(Nib, Nib),      // 8xy1 - OR Vx, Vy
    AndReg(Nib, Nib),     // 8xy2 - AND Vx, Vy
    XorReg(Nib, Nib),     // 8xy3 - XOR Vx, Vy
    AddReg(Nib, Nib),     // 8xy4 - ADD Vx, Vy
    SubReg(Nib, Nib),     // 8xy5 - SUB Vx, Vy
    ShiftRight(Nib, Nib),     // 8xy6 - SHR Vx {, Vy}
    SubNot(Nib, Nib),    // 8xy7 - SUBN Vx, Vy
    ShiftLeft(Nib, Nib),     // 8xyE - SHL Vx {, Vy}
    SkipNotEqualReg(Nib, Nib),     // 9xy0 - SNE Vx, Vy
    LoadIndex(Addr),           // Annn - LD I, addr
    JumpV0(Addr),          // Bnnn - JP V0, addr
    RandomByte(Nib, u8),      // Cxkk - RND Vx, byte
    Draw(Nib,Nib,Nib), // Dxyn - DRW Vx, Vy, nibble
    SkipKeyPressed(Nib),           // Ex9E - SKP Vx
    SkipKeyNotPressed(Nib),          // ExA1 - SKNP Vx
    LoadDelay(Nib),           // Fx07 - LD Vx, DT
    WaitKey(Nib),           // Fx0A - LD Vx, K
    SetDelay(Nib),           // Fx15 - LD DT, Vx
    SetSound(Nib),           // Fx18 - LD ST, Vx
    AddToIndex(Nib),           // Fx1E - ADD I, Vx
    LoadFont(Nib),           // Fx29 - LD F, Vx
    LoadBCD(Nib),           // Fx33 - LD B, Vx
    StoreRegs(Nib),           // Fx55 - LD [I], Vx
    LoadRegs(Nib),           // Fx65 - LD Vx, [I]
}

impl OpCode {
    pub(super) fn decode(code: u16) -> Self {
        use OpCode::*;

        let first_nib = Nib::from((code >> 12) as u8 & 0x0F);
        let x = Nib::from((code >> 8) as u8 & 0x0F);
        let y = Nib::from((code >> 4) as u8 & 0x0F);
        let n = Nib::from(code as u8 & 0x0F);
        let kk = (code & 0x00ff) as u8;
        let addr = Addr::from(code & 0xFFF);

        return match first_nib.nib() {
            0x0 => {
                match code { // todo: do we want these insider matches?
                    0x0000 => NoOp,
                    0x00E0 => ClearScreen,
                    0x00EE => Return,
                    _ => todo!() // handle errors?
                }
            }
            0x1 => Jump(addr),
            0x2 => Call(addr),
            0x3 => SkipEqualByte(x, kk),
            0x4 => SkipNotEqualByte(x, kk),
            0x5 => SkipEqualReg(x, y),
            0x6 => LoadByte(x, kk),
            0x7 => AddByte(x, kk),
            0x8 => match n.nib() {
                0x0 => LoadReg(x, y),
                0x1 => OrReg(x, y),
                0x2 => AndReg(x, y),
                0x3 => XorReg(x, y),
                0x4 => AddReg(x, y),
                0x5 => SubReg(x, y),
                0x6 => ShiftRight(x, y),
                0x7 => SubNot(x, y),
                0xE => ShiftLeft(x, y),
                _ => panic!("Unknown instruction: {:#X}", code),
            },
            0x9 => SkipNotEqualReg(x, y),
            0xA => LoadIndex(addr),
            0xB => JumpV0(addr),
            0xC => RandomByte(x, kk),
            0xD => Draw(x, y, n),
            0xE => match kk {
                0x9E => SkipKeyPressed(x),
                0xA1 => SkipKeyNotPressed(x),
                _ => panic!("Unknown instruction: {:#X}", code),
            },
            0xF => match kk {
                0x07 => LoadDelay(x),
                0x0A => WaitKey(x),
                0x15 => SetDelay(x),
                0x18 => SetSound(x),
                0x1E => AddToIndex(x),
                0x29 => LoadFont(x),
                0x33 => LoadBCD(x),
                0x55 => StoreRegs(x),
                0x65 => LoadRegs(x),
                _ => todo!()
            }
            _ => todo!()
        }
    }
}