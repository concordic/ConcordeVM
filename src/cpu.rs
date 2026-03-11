//! ConcordeVM's CPU.
//!
//! Provides a CPU struct that executes instructions in memory, according to a stack of instruction
//! pointers.
//!
//! Instructions are stored as `Vec<Instruction>`s along with a PC

use crate::{instructions::execute_instruction, instructions::Interrupt, io::ConcordeIO};
use std::rc::Rc;
use crate::memory::*;

use concordeisa::instructions::{self, Instruction};

use log::info;
use std::vec::Vec;

#[derive(Clone)]
pub struct Program{
    pub instructions: Rc<Vec<Instruction>>,
    pub pc: usize
}

impl Default for Program {
    fn default() -> Self {
        Program::new(Vec::new())
    }
}

impl Program {
    /// Create a new empty `ExecutionStack`.
    pub fn new(instructions: Vec<Instruction>) -> Program {
        return Program { instructions: Rc::new(instructions), pc: 0 };
    }

    pub fn fork_to_pc(&self, pc: usize) -> Program {
        return Program { instructions: Rc::clone(&self.instructions), pc: pc }
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
        return self.instructions.to_vec();
    }
}

/// The `CPU` is where instruction reading and execution is handled.
///
/// Contains `Memory`, as well as an `Program`. These are used to read and execute
/// instructions.
#[allow(clippy::upper_case_acronyms)]
pub struct CPU {
    pub memory: Memory,
    io: ConcordeIO,
    pub program: Program,
}

impl CPU {
    /// Create a new `CPU`. Initializes both the memory and stack to be empty.
    pub fn new(memory_size: usize) -> CPU {
        CPU {
            memory: Memory::new(memory_size),
            io: ConcordeIO::new(),
            program: Program::default(),
        }
    }

    pub fn fork_to_pc(self, pc: usize) -> CPU {
        return CPU::with_program(0, self.program.fork_to_pc(pc));
    }

    pub fn with_program(memory_size: usize, program: Program) -> CPU {
        CPU {
            memory: Memory::new(memory_size),
            io: ConcordeIO::new(),
            program: program
        }
    }

    pub fn get_memory_mut(&mut self) -> &mut Memory {
        return &mut self.memory;
    }

    pub fn load_program(&mut self, program: Program) {
        self.program = program;
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
