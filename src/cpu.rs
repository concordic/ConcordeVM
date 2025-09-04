// This is where code gets read and executed

use crate::memory::*;
use crate::instructions::*;

use std::vec::Vec;

pub struct CPU {
    memory: Memory,
    stack: Vec<Symbol>,
    instruction_cache: Vec<Instruction>,
    instruction_index: usize,
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            memory: Memory::new(),
            stack: Vec::new(),
            instruction_cache: Vec::new(),
            instruction_index: 0,
        }
    }

    pub fn cycle(&mut self) {
        let next_instruction = self.instruction_cache[self.instruction_index];
        executeInstruction(next_instruction, self.memory);
        self.instruction_index += 1;
    }
}
