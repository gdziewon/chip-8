use std::ops::{Index, IndexMut};

use super::opcode::Nib;

const NUM_REGISTERS: usize = 16;
const FLAG_REGISTER: usize = 0xF;

pub struct Registers {
    regs: [u8; NUM_REGISTERS]
}

impl Registers {
    pub fn new() -> Self {
        Registers { regs: [0; NUM_REGISTERS] }
    }

    pub fn v0(&self) -> u8 {
        self.regs[0]
    }

    pub fn set_flag(&mut self, val: u8) {
        self.regs[FLAG_REGISTER] = val;
    }

    // fn flag_reg(&self) -> &u8 {
    //     &self.regs[FLAG_REGISTER]
    // }

    // fn flag_reg_mut(&mut self) -> &mut u8 {
    //     &mut self.regs[FLAG_REGISTER]
    // }
}

impl Index<Nib> for Registers {
    type Output = u8;

    fn index(&self, index: Nib) -> &Self::Output {
        &self.regs[index.value() as usize]
    }
}

impl IndexMut<Nib> for Registers{
    fn index_mut(&mut self, index: Nib) -> &mut Self::Output {
        &mut self.regs[index.value() as usize]
    }
}