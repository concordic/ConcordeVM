//! ConcordeVM's CPU.
//!
//! Provides a CPU struct that executes instructions in memory, according to a stack of instruction
//! pointers.
//!
//! Instructions are stored as `Vec<Instruction>`s under symbols in memory. 

use crate::{instructions::execute_instruction, io::ConcordeIO};
use crate::memory::*;

use concordeisa::{instructions::Instruction, memory::Symbol};

use log::info;
use std::vec::Vec;

/// `ExecutionPointer`s represent a location in memory where code is being executed.
///
/// Contains the symbol under which the instructions are stored, as well as the index of the
/// instruction currently being executed.
#[derive(Clone, Eq, PartialEq)]
pub struct ExecutionPointer {
    pub symbol: Symbol,
    pub index: usize,
}

/// The `ExecutionStack` is the stack of the CPU. It stores `ExecutionPointer`s to every block of
/// code being executed at any moment. 
#[derive(Clone)]
pub struct ExecutionStack(Vec<ExecutionPointer>);

impl Default for ExecutionStack {
    fn default() -> Self {
        ExecutionStack::new()
    }
}

impl ExecutionStack {
    /// Create a new empty `ExecutionStack`.
    pub fn new() -> ExecutionStack {
        ExecutionStack(Vec::new())
    }

    /// Delete everything in the stack.
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Get the top pointer on the stack. Returns None if the stack is empty.
    pub fn top(&self) -> Option<&ExecutionPointer>{
        self.0.last()
    }

    /// Increment the index of the top pointer on the stack.
    pub fn increment(&mut self) {
        self.0.last_mut().unwrap().index += 1;
    }

    /// Jump execution to a given symbol. Will not error, even if the symbol is undefined.
    pub fn jump(&mut self, target: &Symbol) {
        info!("Jumped to {}!", target.0);
        self.0.push(ExecutionPointer { symbol: target.clone(), index: 0 });
    }

    /// Return execution to the previous location. Will not error.
    pub fn ret(&mut self) {
        info!("Returned!");
        self.0.pop();
    }

    /// if goto is valid, sets symbol to new location and index to 0 to begin at top of instruction stack
    pub fn goto(&mut self, target: &Symbol) {
        if let Some(pointer) = self.0.last_mut() {
            info!("Goto {}!", target.0);//goto symbol
            pointer.symbol = target.clone();
            pointer.index = 0;
        }
    }

    pub fn dump(&self) -> Vec<ExecutionPointer> {
        self.0.clone()
    }
}

/// The `CPU` is where instruction reading and execution is handled.
///
/// Contains `Memory`, as well as an `ExecutionStack`. These are used to read and execute
/// instructions.
#[allow(clippy::upper_case_acronyms)]
pub struct CPU {
    memory: Memory,
    io: ConcordeIO,
    stack: ExecutionStack,
}

impl CPU {
    /// Create a new `CPU`. Initializes both the memory and stack to be empty.
    pub fn new() -> CPU {
        CPU {
            memory: Memory::new(),
            io: ConcordeIO::new(),
            stack: ExecutionStack::new(),
        }
    }
    
    /// Load instructions into memory at a given symbol.
    pub fn load_instructions(&mut self, instructions: &Vec<Instruction>, symbol: &Symbol) {
        self.memory.write(symbol, Data::new(instructions));
        info!("Loaded {} instructions into symbol {}", instructions.len(), symbol.0);
    }

    /// Get the CPU ready to start executing code. Clears the stack and jumps to the entrypoint.
    pub fn init_execution(&mut self, entrypoint: &Symbol) {
        self.stack.clear();
        self.stack.jump(entrypoint);
    }

    /// Complete one CPU cycle. Returns false iff the stack is empty. Returns an error if something
    /// goes wrong during execution. Returns true otherwise.
    ///
    /// Each CPU cycle does the following:
    ///   - Checks if the stack is empty. If it is, return false. If not, continue.
    ///   - Reads the instructions that the `ExecutionPointer` at the top of the stack points to.
    ///   - If we're done execution there, return from that block, and return true. 
    ///   - Otherwise, read the instruction at the given index and execute it.
    ///   - If the instruction errors, return the error. Otherwise, return true.
    ///
    /// One CPU cycle does not necessarily map to one instruction, as a CPU cycle is used every time
    /// we pop an execution pointer off of the stack when we are done executing those instructions. This is
    /// technically equivalent to every instruction vector having a return instruction tacked on at
    /// the end, but isn't handled the same way.
    pub fn cycle(&mut self) -> Result<bool, String> {
        if let Some(exec_pointer) = self.stack.top() {
            info!("Currently executing code at symbol [{}], index {}", exec_pointer.symbol.0, exec_pointer.index);
            let instruction_vec = self.memory.read_typed::<Vec<Instruction>>(&exec_pointer.symbol)?;
            // This execution pointer has reached the end of it's code, so we can return
            if instruction_vec.len() <= exec_pointer.index {
                info!("Execution pointer at symbol {} has reached the end of it's code at index {}!", exec_pointer.symbol.0, exec_pointer.index);
                self.stack.ret();
                if self.stack.top().is_none() {
                    info!("CPU stack is empty!");
                    return Ok(false);
                }
                self.stack.increment();
            } else {
                let instruction = &instruction_vec[exec_pointer.index].clone();
                execute_instruction(instruction, &mut self.memory, &mut self.io, &mut self.stack)?;
            }
            Ok(true)
        }
        else {
            info!("CPU Stack is empty!");
            Ok(false)
        }
    }

    /// Get a clone of the memory for debugging.
    pub fn get_memory(&self) -> Memory {
        self.memory.clone()
    }

    /// Get a clone of the stack for debugging.
    pub fn get_stack(&self) -> ExecutionStack {
        self.stack.clone()
    }
}

impl Default for CPU {
   fn default() -> Self {
       CPU::new()
   } 
}
