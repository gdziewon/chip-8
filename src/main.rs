use rand::Rng;

struct Chip8 {
    // Registers
    V: [u8; 8], // 8 general purpose 8-bit registers
    I: u16, // 16-bit address register

    // Timers  - count down at 60hz to 0
    DT: u8, // delay timer
    ST: u8, // sound timer

    PC: u16, // program counter
    SP: u8, // stack pointer
    stack: [u16; 16], // 16 16-bit stack fields

    display: [[bool; 32]; 64],
}

impl Chip8 {
    fn new() -> Self {
        Chip8 {
            V: [0x00; 8],
            I: 0x0000,
            DT: 0x00,
            ST: 0x00,
            PC: 0x200,
            SP: 0x00,
            stack: [0x0000; 16],
            display: [[false; 32]; 64]
        }
    }
    fn execute( &mut self, opCode: u16, mem: &mut [u8]) {
        match opCode >> 12 { // match by most significant 4bits
            0x0 => match opCode {
                0x00ee => { // RET
                    self.PC = self.stack[self.SP as usize];
                    self.SP -= 1;
                }
                0x0e0 => { // CLS
                    self.display = [[false; 32]; 64];
                }
                0x0000  => {
                    println!("NOP! PC: {:#03x}", self.PC);
                } // NOP
                _ => { // CALL (0nnn)
                    self.SP += 1;
                    self.stack[self.SP as usize] = self.PC;
                    let addr = Chip8::get_addr(opCode);
                    self.PC = addr;
                }
            }
            0x1 => { // JP addr
                let addr = Chip8::get_addr(opCode);
                self.PC = addr;
            }
            0x2 => { // CALL addr
                self.SP += 1;
                self.stack[self.SP as usize] = self.PC;
                let addr = Chip8::get_addr(opCode);
                self.PC = addr;
            }
            0x3 => { // SE Vx, byte
                let vx = Chip8::get_vx(opCode);
                let data = Chip8::get_data_byte(opCode);
                if self.V[vx] == data {
                    self.I += 2;
                }
            }
            0x4 => { // SNE Vx, byte
                let vx = Chip8::get_vx(opCode);
                let data = Chip8::get_data_byte(opCode);
                if self.V[vx] != data {
                    self.PC += 2;
                }
            }
            0x5 => { // SE Vx, Vy
                let vx = Chip8::get_vx(opCode);
                let vy = Chip8::get_vy(opCode);
                if self.V[vx] == self.V[vy] {
                    self.PC += 2;
                }
            }
            0x6 => { // LD Vx, byte
                let vx = Chip8::get_vx(opCode);
                let data = Chip8::get_data_byte(opCode);
                self.V[vx] = data;
            }
            0x7 => { // ADD Vx, byte
                let vx = Chip8::get_vx(opCode);
                let data = Chip8::get_data_byte(opCode);
                self.V[vx] += data;
            }
            0x8 => {
                let vx = Chip8::get_vx(opCode);
                let vy = Chip8::get_vy(opCode);
                match opCode & 0x000f {
                    0x0 => { // LD Vx, Vy
                        self.V[vx] = self.V[vy];
                    },
                    0x1 => { // OR Vx, Vy
                        self.V[vx] = self.V[vx] | self.V[vy];
                    },
                    0x2 => { // AND Vx, Vy
                        self.V[vx] = self.V[vx] & self.V[vy];
                    },  
                    0x3 => { // XOR Vx, Vy
                        self.V[vx] = self.V[vx] ^ self.V[vy];
                    }
                    0x4 => { // ADD Vx, Vy
                        self.V[8] = if self.V[vx] as u16 + self.V[vy] as u16 > 0xff { 1 } else { 0 };
                        self.V[vx] = self.V[vx].wrapping_add(self.V[vy]);
                    }
                    0x5 => { // SUB Vx, Vy
                        self.V[8] = if self.V[vx] >= self.V[vy] { 1 } else { 0 };
                        self.V[vx] = self.V[vx].wrapping_sub(self.V[vy]);
                    }
                    0x6 => { // SHR Vx {, Vy}
                        self.V[8] = self.V[vx] & 1;
                        self.V[vx] >>= 1;
                    }
                    0x7 => { // SUBN Vx, Vy
                        self.V[8] = if self.V[vx] <= self.V[vy] { 1 } else { 0 };
                        self.V[vx] = self.V[vy].wrapping_sub(self.V[vx]);
                    }
                    0xe => { // SHL Vx {, Vy}
                        self.V[8] = self.V[vx] >> 7;
                        self.V[vx] <<= 1;
                    }
                    _ => ()
                }
            }
            0x9 => {
                let vx = Chip8::get_vx(opCode);
                let vy = Chip8::get_vy(opCode);
                if self.V[vx] != self.V[vy] {
                    self.PC += 2;
                }
            }
            0xa => { // LD I, addr
                let addr = Chip8::get_addr(opCode);
                self.I = addr;
            }
            0xb => { // JP V0, addr
                let addr = Chip8::get_addr(opCode);
                self.PC = addr + self.V[0] as u16;
            }
            0xc => { // RND Vx, byte
                let vx = Chip8::get_vx(opCode);
                let data = Chip8::get_data_byte(opCode);
                let rnd : u8 = rand::thread_rng().gen();
                self.V[vx] = data & rnd;
            }
            0xd => { // DRW Vx, Vy, nibble
                let vx = Chip8::get_vx(opCode);
                let vy = Chip8::get_vy(opCode);
                let n = opCode & 0xf;
                for i in 0..n {
                    let byte = mem[self.I as usize];
                    for j in 0..8 {
                        self.display[((self.V[vx] % 64) + j) as usize][((self.V[vy] % 32) + i as u8) as usize] ^= ((byte >> (7 - j)) & 1) != 0;
                        println!("{}", ((byte >> (7 - j)) & 1));
                    }
                    self.I += 1;
                }
            }
            0xe => match opCode & 0x00ff {
                0x9e => { // SKP Vx
                    // key press
                }
                0xa1 => { // SKNP Vx
                    // key press
                }
                _ => ()
            }
            0xf => {
                let vx = Chip8::get_vx(opCode);
                match opCode & 0x00ff {
                    0x07 => { // LD Vx, DT
                        self.V[vx] = self.DT;
                    }
                    0x0a => { // LD Vx, K
                        // key press
                    }
                    0x15 => { // LD DT, Vx
                        self.DT = self.V[vx];
                    }
                    0x18 => { // LD ST, Vx
                        self.ST = self.V[vx];
                    }
                    0x1e => { // ADD I, Vx
                        self.I += self.V[vx] as u16;
                    }
                    0x29 => { // LD F, Vx
                        // display
                    }
                    0x33 => { // LD B, Vx

                    }
                    0x55 => { // LD [I], Vx
                        for i in 0..vx {
                            mem[self.I as usize] = self.V[i];
                            self.I += 1;
                        }
                        self.I += 1;
                    }
                    0x65 => {
                        for i in 0..vx {
                            self.V[i] = mem[self.I as usize];
                            self.I += 1;
                        }
                        self.I += 1;
                    }
                    _ => ()
                }
            }
            _ => ()
        }
        self.PC += 2;
    }

    fn run( &mut self, mem: &mut [u8] ) {
        while self.PC < 550 {
            let instruction: u16 = (mem[self.PC as usize] as u16) << 8 |
                (mem[(self.PC + 1) as usize] as u16);
            self.execute(instruction, mem);
            for i in 0..32 {
                for j in 0..64 {
                    print!("{}", self.display[j][i] as u8);
                }
                println!();
            }
        }
    }

    fn get_vx(opCode: u16) -> usize{
        ((opCode >> 8) & 0x000f) as usize
    }

    fn get_vy(opCode: u16) -> usize {
        ((opCode >> 4) & 0x000f) as usize
    }

    fn get_data_byte(opCode: u16) -> u8 {
        (opCode & 0x00ff) as u8
    }

    fn get_addr(opCode: u16) -> u16 {
        opCode & 0x0fff
    }
}


fn main() {
    
    let mut chip8 = Chip8::new();
    let mut mem: [u8; 1024 * 4] = [0; 1024 * 4];

    mem[0x200] = 0xa3;
    mem[0x201] = 0x00;
    mem[0x202] = 0xd0;
    mem[0x203] = 0x05;

    // 0 
    mem[0x300] = 0xf0;
    mem[0x301] = 0x90;
    mem[0x302] = 0x90;
    mem[0x303] = 0x90;
    mem[0x304] = 0xf0;

    chip8.run(&mut mem);
}
