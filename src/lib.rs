//! ConcordeVM's library version.

mod cpu;
pub use cpu::{
    CPU,
    ExecutionPointer,
    CallTree,
};

mod memory;
pub use memory::{
    Memory,
};

mod io;
mod instructions;

#[macro_use]
mod errors;

#[cfg(test)]
mod tests;
