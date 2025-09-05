//! ConcordeVM's CPU.
//!
//! Provides a CPU struct that executes instructions in memory, according to a stack of instruction
//! pointers.
//!
//! Instructions are stored as `Vec<Instruction>`s under symbols in memory. 

use crate::instructions::{Instruction, execute_instruction};
use crate::memory::*;

use log::info;
use std::vec::Vec;

// `ExecutionPointer`s represent a location in memoty where code is being executed.
//
// Contains the symbol under which the instructions are stored, as well as the index of the
// instruction currently being executed.
struct ExecutionPointer {
    symbol: Symbol,
    index: usize,
}

// The `CPU` is where instruction reading and execution is handled.
//
// Contains `Memory`, as well as a stack of `ExecutionPointer`s.
//
// Currently, the stack does nothing of value, since we do not support branching or functions.
// However, once those are implemented, the stack will be used to keep track of the return location
// for every layer of function calls or branches.
#[allow(clippy::upper_case_acronyms)]
pub struct CPU {
    memory: Memory,
    stack: Vec<ExecutionPointer>,
}

impl CPU {
    // Create a new `CPU`. Initializes both the memory and stack to be empty.
    pub fn new() -> CPU {
        CPU {
            memory: Memory::new(),
            stack: Vec::new(),
        }
    }

    // Complete one CPU cycle. Returns false iff the stack is empty. Returns an error if something
    // goes wrong during execution. Returns true otherwise.
    //
    // One CPU cycle does not necessarily map to one instruction, as a CPU cycle is used every time
    // we pop an `ExecutionPointer` off of the stack when we are done executing those
    // instructions. However, excluding that, each cycle executes one instruction.
    pub fn cycle(&mut self) -> Result<bool, String> {
        if let Some(exec_pointer) = self.stack.last_mut() {
            info!("Currently executing code at symbol [{}], index {}", exec_pointer.symbol.0, exec_pointer.index);
            let instruction_vec = self.memory.read_typed::<Vec<Instruction>>(&exec_pointer.symbol)?;
            // This exec pointer has reached the end of it's code, so we can pop it off
            if instruction_vec.len() <= exec_pointer.index {
                info!("Exec pointer at symbol {} has reached the end of it's code at index {}!", exec_pointer.symbol.0, exec_pointer.index);
                self.stack.pop();
                return Ok(true);
            }
            let instruction = &instruction_vec[exec_pointer.index].clone();
            execute_instruction(instruction, &mut self.memory)?;
            exec_pointer.index += 1;
            Ok(true)
        }
        else {
            info!("CPU Stack is empty!");
            Ok(false)
        }
    }

    // Load instructions into memory and add an `ExecutionPointer` for them to the stack.
    pub fn load_instructions(&mut self, instructions: &Vec<Instruction>, symbol: &Symbol) {
        self.memory.write(symbol, Data::new(instructions));
        self.stack.push(ExecutionPointer { symbol: symbol.clone(), index: 0 });
        info!("Loaded {} instructions into symbol {}", instructions.len(), symbol.0);
    }
}
