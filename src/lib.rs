//! ConcordeVM's library version.

mod cpu;
pub use cpu::{
    CPU,
    ExecutionPointer,
    ExecutionStack,
};

mod memory;
pub use memory::{
    Data,
    Memory,
};

mod instructions;

#[macro_use]
mod errors;
