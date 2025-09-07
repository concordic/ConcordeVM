//! ConcordeVM's library version.

mod cpu;
pub use cpu::CPU; 

#[cfg(feature = "debug-visibility")]
pub use cpu::{
    ExecutionPointer,
    ExecutionStack,
};

mod memory;

#[cfg(feature = "debug-visibility")]
pub use memory::{
    Data,
    Memory,
};

mod instructions;

#[macro_use]
mod errors;
