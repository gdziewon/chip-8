
use modular_bitfield::prelude::*;


// pub struct OpCode {
//     pub code: u16,
// }

// impl OpCode {
//     pub fn new(code: u16) -> Self { OpCode { code }}
//     pub fn vx (&self) -> usize { ((self.code >> 8) & 0x000f) as usize }
//     pub fn vy (&self) -> usize { ((self.code >> 4) & 0x000f) as usize }
//     pub fn nibble (&self) -> u8 { (self.code & 0x000f) as u8 }
//     pub fn byte (&self) -> u8 { (self.code & 0x00ff) as u8 }
//     pub fn addr (&self) -> u16 { self.code & 0x0fff }
// }

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
    NOP,                 // NOP
    //SYS(Addr),         // 0nnn - SYS addr -> not used in modern interpreters
    CLS,                 // 00E0 - CLS
    RET,                 // 00EE - RET
    JPa(Addr),           // 1nnn - JP addr
    CALLa(Addr),         // 2nnn - CALL addr
    SEvb(Nib, u8),       // 3xkk - SE Vx, byte
    SNEvb(Nib, u8),      // 4xkk - SNE Vx, byte
    SEvv(Nib, Nib),      // 5xy0 - SE Vx, Vy
    LDvb(Nib, u8),      // 6xkk - LD Vx, byte
    ADDvb(Nib, u8),     // 7xkk - ADD Vx, byte
    LDvv(Nib, Nib),      // 8xy0 - LD Vx, Vy
    ORvv(Nib, Nib),      // 8xy1 - OR Vx, Vy
    ANDvv(Nib, Nib),     // 8xy2 - AND Vx, Vy
    XORvv(Nib, Nib),     // 8xy3 - XOR Vx, Vy
    ADDvv(Nib, Nib),     // 8xy4 - ADD Vx, Vy
    SUBvv(Nib, Nib),     // 8xy5 - SUB Vx, Vy
    SHRvv(Nib, Nib),     // 8xy6 - SHR Vx {, Vy}
    SUBNvv(Nib, Nib),    // 8xy7 - SUBN Vx, Vy
    SHLvv(Nib, Nib),     // 8xyE - SHL Vx {, Vy}
    SNEvv(Nib, Nib),     // 9xy0 - SNE Vx, Vy
    LDa(Addr),           // Annn - LD I, addr
    JPva(Addr),          // Bnnn - JP V0, addr
    RNDvb(Nib, u8),      // Cxkk - RND Vx, byte
    DRWvvn(Nib,Nib,Nib), // Dxyn - DRW Vx, Vy, nibble
    SKPv(Nib),           // Ex9E - SKP Vx
    SKNPv(Nib),          // ExA1 - SKNP Vx
    LDvd(Nib),           // Fx07 - LD Vx, DT
    LDvk(Nib),           // Fx0A - LD Vx, K
    LDdv(Nib),           // Fx15 - LD DT, Vx
    LDsv(Nib),           // Fx18 - LD ST, Vx
    ADDi(Nib),           // Fx1E - ADD I, Vx
    LDfv(Nib),           // Fx29 - LD F, Vx
    LDbv(Nib),           // Fx33 - LD B, Vx
    LDiv(Nib),           // Fx55 - LD [I], Vx
    LDvi(Nib),           // Fx65 - LD Vx, [I]
}

impl OpCode {
    pub fn decode(code: u16) -> Self {
        use OpCode::*;

        let first_nib = Nib::from((code >> 12) as u8 & 0x0F);
        let x = Nib::from((code >> 8) as u8 & 0x0F);
        let y = Nib::from((code >> 4) as u8 & 0x0F);
        let n = Nib::from(code as u8 & 0x0F);
        let kk = (code & 0x00ff) as u8;
        let addr = Addr::from(code & 0xFFF);

        return match first_nib.nib() {
            0x0 => {
                match code {
                    0x0000 => NOP,
                    0x00E0 => CLS,
                    0x00EE => RET,
                    _ => todo!() // handle errors?
                }
            }
            0x1 => JPa(addr),
            0x2 => CALLa(addr),
            0x3 => SEvb(x, kk),
            0x4 => SNEvb(x, kk),
            0x5 => SEvv(x, y),
            0x6 => LDvb(x, kk),
            0x7 => ADDvb(x, kk),
            0x8 => match n.nib() {
                0x0 => LDvv(x, y),
                0x1 => ORvv(x, y),
                0x2 => ANDvv(x, y),
                0x3 => XORvv(x, y),
                0x4 => ADDvv(x, y),
                0x5 => SUBvv(x, y),
                0x6 => SHRvv(x, y),
                0x7 => SUBNvv(x, y),
                0xE => SHLvv(x, y),
                _ => panic!("Unknown 8xy* instruction: {:#X}", code),
            },
            0x9 => SNEvv(x, y),
            0xA => LDa(addr),
            0xB => JPva(addr),
            0xC => RNDvb(x, kk),
            0xD => DRWvvn(x, y, n),
            0xE => match kk {
                0x9E => SKPv(x),
                0xA1 => SKNPv(x),
                _ => todo!(),
            },
            0xF => match kk {
                0x07 => LDvd(x),
                0x0A => LDvk(x),
                0x15 => LDdv(x),
                0x18 => LDsv(x),
                0x1E => ADDi(x),
                0x29 => LDfv(x),
                0x33 => LDbv(x),
                0x55 => LDiv(x),
                0x65 => LDvi(x),
                _ => todo!()
            }
            _ => todo!()
        }
    }
}