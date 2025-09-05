// This is where code gets read and executed

use crate::instructions::{Instruction, execute_instruction};
use crate::memory::*;

use log::info;
use std::vec::Vec;

struct ExecutionPointer {
    symbol: Symbol,
    index: usize,
}

#[allow(clippy::upper_case_acronyms)]
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

    pub fn load_instructions(&mut self, instructions: &Vec<Instruction>, symbol: &Symbol) {
        self.memory.write(symbol, Data::new(instructions));
        self.stack.push(ExecutionPointer { symbol: symbol.clone(), index: 0 });
        info!("Loaded {} instructions into symbol {}", instructions.len(), symbol.0);
    }
}
