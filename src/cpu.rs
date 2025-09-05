// This is where code gets read and executed

use crate::errors::log_and_return_err;
use crate::instructions::{Instruction, execute_instruction};
use crate::memory::*;

use log::{info, warn, error};
use std::vec::Vec;

struct ExecutionPointer {
    symbol: Symbol,
    index: usize,
}

pub struct CPU {
    memory: Memory,
    stack: Vec<ExecutionPointer>,
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            memory: Memory::new(),
            stack: Vec::new(),
        }
    }

    // Complete one CPU cycle. This executes one instruction, and returns false iff the stack is
    // empty and there is nothing more to execute
    pub fn cycle(&mut self) -> Result<bool, String> {
        if let Some(exec_pointer) = self.stack.last() {
            match self.memory.read::<Vec<Instruction>>(&exec_pointer.symbol) {
                Ok(instruction_vec) => {
                    // This exec pointer has reached the end of it's code, so we can pop it off
                    if instruction_vec.len() >= exec_pointer.index {
                        info!("Exec pointer at symbol {} has reached the end of it's code!", exec_pointer.symbol.0);
                        self.stack.pop();
                        return Ok(true);
                    }
                    let instruction = &instruction_vec[exec_pointer.index].clone();
                    execute_instruction(instruction, &mut self.memory);
                    Ok(true)
                }
                Err(e) => log_and_return_err!("Couldn't get instruction data due to previous error:\n\t => {}", e)
            }
        }
        else {
            println!("CPU Stack is empty!");
            Ok(false)
        }
    }
}
