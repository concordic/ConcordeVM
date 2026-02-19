//! ConcordeVM's library version.

mod cpu;
pub use cpu::{
    CPU,
    Program,
};

mod memory;
pub use memory::{
    Memory,
};

mod io;
mod instructions;
pub use instructions::{
    Interrupt
};

#[macro_use]
mod errors;

#[cfg(test)]
mod tests;
