//! ConcordeVM's CPU.
//!
//! Provides a CPU struct that executes instructions in memory, according to a stack of instruction
//! pointers.
//!
//! Instructions are stored as `Vec<Instruction>`s along with a PC

use crate::{instructions::execute_instruction, instructions::Interrupt, io::ConcordeIO};
use crate::memory::*;

use concordeisa::{instructions::Instruction};

use log::info;
use std::vec::Vec;

#[derive(Clone)]
pub struct Program{
    instructions: Vec<Instruction>,
    pc: usize
}

impl Default for Program {
    fn default() -> Self {
        Program::new()
    }
}

impl Program {
    /// Create a new empty `ExecutionStack`.
    pub fn new() -> Program {
        return Program { instructions: Vec::new(), pc: 0 };
    }

    pub fn get_instruction(&self) -> &Instruction{
        return &self.instructions[self.pc];
    }

    /// Increment the index of the top pointer on the stack.
    pub fn increment(&mut self) {
        self.pc += 1;
    }

    /// Jump execution to a given symbol. Will not error, even if the symbol is undefined.
    pub fn jump(&mut self, target: usize) {
        info!("Jumped to {}!", target);
        self.pc = target;
    }

    pub fn dump(&self) -> Vec<Instruction> {
        return self.instructions.clone();
    }
}

/// The `CPU` is where instruction reading and execution is handled.
///
/// Contains `Memory`, as well as an `Program`. These are used to read and execute
/// instructions.
#[allow(clippy::upper_case_acronyms)]
pub struct CPU {
    memory: Memory,
    io: ConcordeIO,
    program: Program,
}

impl CPU {
    /// Create a new `CPU`. Initializes both the memory and stack to be empty.
    pub fn new(memory_size: usize) -> CPU {
        CPU {
            memory: Memory::new(memory_size),
            io: ConcordeIO::new(),
            program: Program::new(),
        }
    }
    
    /// Load instructions into memory at a given symbol.
    pub fn load_instructions(&mut self, instructions: &Vec<Instruction>) {
            self.program.instructions = instructions.clone();
            self.program.pc = 0;
        info!("Loaded {} instructions", instructions.len());
    }

    /// Get the CPU ready to start executing code. Clears the stack and jumps to the entrypoint.
    pub fn init_execution(&mut self, entrypoint: usize) {
        self.program.jump(entrypoint);
    }

    // Runs until an interrupt is triggered
    pub fn run(&mut self) -> Result<Interrupt, String> {
        while self.program.pc < self.program.instructions.len() {
            match self.cycle()? {
                Interrupt::Ok => {},
                Interrupt::EOF => {return Ok(Interrupt::EOF);},
                interrupt @ _ => {return Ok(interrupt)}
            };
        };
        return Ok(Interrupt::Ok);
    }

    // Run a single FDE cycle
    pub fn cycle(&mut self) -> Result<Interrupt, String> {
        if self.program.pc < self.program.instructions.len() {
            return execute_instruction(&mut self.memory, &mut self.io, &mut self.program);
        }
        info!("Reached end of program!");
        Ok(Interrupt::Ok)
    }

    /// Get a clone of the memory for debugging.
    pub fn get_memory(&self) -> Memory {
        self.memory.clone()
    }

    /// Get a clone of the stack for debugging.
    pub fn get_stack(&self) -> Program {
        self.program.clone()
    }

    pub fn extend_memory(&mut self, n: usize){
        self.memory.extend_memory(n);
    }
}

impl Default for CPU {
   fn default() -> Self {
       CPU::new(0)
   } 
}
