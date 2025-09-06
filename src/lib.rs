//! ConcordeVM's library version.

pub use concordeisa;
pub mod cpu;

mod instructions;
mod memory;

#[macro_use]
mod errors;
