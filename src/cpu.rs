//! ConcordeVM's CPU.
//!
//! Provides a CPU struct that executes instructions in memory, according to a tree of instruction
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
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ExecutionPointer {
    pub parent: Option<Box<ExecutionPointer>>,
    pub children: Vec<Box<ExecutionPointer>>,
    pub symbol: Symbol,
    pub index: usize,
}

/// The `CallTree` is the tree of the CPU. It stores `ExecutionPointer`s to every block of
/// code being executed at any moment. 
#[derive(Clone)]
pub struct CallTree {
    // process roots for fully independent threads (daemons and such)
    roots: Vec<ExecutionPointer>,
    // every node thats currently not blocked 
    leaves: Vec<ExecutionPointer>
}

impl Default for CallTree {
    fn default() -> Self {
        CallTree::new()
    }
}

impl CallTree {
    /// Create a new empty `CallTree`.
    pub fn new() -> CallTree {
        CallTree {
            roots: Vec::new(),
            leaves: Vec::new(),
        }
    }

    /// Delete everything in the tree.
    pub fn clear(&mut self) {
        self.roots.clear();
        self.leaves.clear();
    }
    
    /// Spawn a new node
    fn spawn(&mut self, symbol: Symbol, parent: Option<Box<ExecutionPointer>>) {
        let push = parent.is_none();
        let new_node = ExecutionPointer {
            parent: parent,
            children: Vec::new(),
            symbol: symbol,
            index: 0,
        };
        if push {
            self.roots.push(new_node);
        }
        self.leaves.push(new_node);
    }
}

/// The `CPU` is where instruction reading and execution is handled.
///
/// Contains `Memory`, as well as an `CallTree`. These are used to read and execute
/// instructions.
#[allow(clippy::upper_case_acronyms)]
pub struct CPU {
    memory: Memory,
    io: ConcordeIO,
    tree: CallTree,
}

impl CPU {
    /// Create a new `CPU`. Initializes both the memory and tree to be empty.
    pub fn new() -> CPU {
        CPU {
            memory: Memory::new(),
            io: ConcordeIO::new(),
            tree: CallTree::new(),
        }
    }
    
    /// Load instructions into memory at a given symbol.
    pub fn load_instructions(&mut self, instructions: &Vec<Instruction>, symbol: &Symbol) {
        self.memory.write(symbol, Data::new(instructions));
        info!("Loaded {} instructions into symbol {}", instructions.len(), symbol.0);
    }

    /// Get the CPU ready to start executing code. Clears the tree and jumps to the entrypoint.
    pub fn init_execution(&mut self, entrypoint: &Symbol) {
        self.tree.clear();
        self.tree.jump(entrypoint);
    }

    /// Complete one CPU cycle. Returns false iff the tree is empty. Returns an error if something
    /// goes wrong during execution. Returns true otherwise.
    ///
    /// Each CPU cycle does the following:
    ///   - Checks if the tree is empty. If it is, return false. If not, continue.
    ///   - Reads the instructions that the `ExecutionPointer` at the top of the tree points to.
    ///   - If we're done execution there, return from that block, and return true. 
    ///   - Otherwise, read the instruction at the given index and execute it.
    ///   - If the instruction errors, return the error. Otherwise, return true.
    ///
    /// One CPU cycle does not necessarily map to one instruction, as a CPU cycle is used every time
    /// we pop an execution pointer off of the tree when we are done executing those instructions. This is
    /// technically equivalent to every instruction vector having a return instruction tacked on at
    /// the end, but isn't handled the same way.
    pub fn cycle(&mut self) -> Result<bool, String> {
        if let Some(exec_pointer) = self.tree.top() {
            info!("Currently executing code at symbol [{}], index {}", exec_pointer.symbol.0, exec_pointer.index);
            let instruction_vec = self.memory.read_typed::<Vec<Instruction>>(&exec_pointer.symbol)?;
            // This execution pointer has reached the end of it's code, so we can return
            if instruction_vec.len() <= exec_pointer.index {
                info!("Execution pointer at symbol {} has reached the end of it's code at index {}!", exec_pointer.symbol.0, exec_pointer.index);
                self.tree.ret();
                if self.tree.top().is_none() {
                    info!("CPU tree is empty!");
                    return Ok(false);
                }
                self.tree.increment();
            } else {
                let instruction = &instruction_vec[exec_pointer.index].clone();
                execute_instruction(instruction, &mut self.memory, &mut self.io, &mut self.tree)?;
            }
            Ok(true)
        }
        else {
            info!("CPU tree is empty!");
            Ok(false)
        }
    }

    /// Get a clone of the memory for debugging.
    pub fn get_memory(&self) -> Memory {
        self.memory.clone()
    }

    /// Get a clone of the tree for debugging.
    pub fn get_tree(&self) -> CallTree {
        self.tree.clone()
    }
}

impl Default for CPU {
   fn default() -> Self {
       CPU::new()
   } 
}
