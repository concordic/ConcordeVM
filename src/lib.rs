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

mod io;
mod instructions;

#[macro_use]
mod errors;

#[cfg(test)]
mod tests;
