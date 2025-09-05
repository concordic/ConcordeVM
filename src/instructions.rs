//! ConcordeVM's instruction set.
//!
//! Provides an enum of possible instructions, as well as a means to map those instructions to
//! functions to execute them.
//!
//! Instructions should generally be as simple as necessary. Unless there is a major advantage to
//! doing so, instructions should not be possible to construct out of other instructions. (eg. no
//! need to implement an instruction that adds 3 numbers together instead of 2, since you can do
//! that with the 2 number addition just fine)

use crate::memory::*;

use log::error;

// The `Instruction` enum provides a set of instructions that the ConcordeVM can execute.
//
// Each instruction also has its parameters, which can be either `Symbol`s, or Rust primitives.
// Instruction names should be self-explanatory, since we want to avoid having excessively complex
// instructions anyways.
//
// We may move away from allowing Rust primitves, since they currently only function to load
// literals into memory. We can, in theory, replace this with an external loader that runs when
// code is loaded into memory.
#[derive(Clone)]
pub enum Instruction {
    // Immediate writes
    WriteStringToSymbol(Symbol, String),
    WriteIntToSymbol(Symbol, i64),

    // Memory management
    CopySymbol(Symbol, Symbol),
    
    // Arithmetic
    AddSymbols(Symbol, Symbol, Symbol),

    // I/O
    PrintSymbol(Symbol),

    // Misc.
    NoOp(),
}

// Execute the given instruction, returning an error if something goes wrong. (eg. division by
// zero, or accessing invalid memory)
//
// Currently, each instruction from the enum maps to a function of the same name in a `match` statement. There
// may be a better way to do this that's more extensible.
pub fn execute_instruction(instruction: &Instruction, memory: &mut Memory) -> Result<(), String> {
    match instruction {
        Instruction::WriteStringToSymbol(symbol, value) => write_string_to_symbol(memory, symbol, value),
        Instruction::WriteIntToSymbol(symbol, value) => write_int_to_symbol(memory, symbol, value),
        Instruction::CopySymbol(source, dest) => copy_symbol(memory, source, dest),
        Instruction::AddSymbols(a, b, dest) => add_symbols(memory, a, b, dest),
        Instruction::PrintSymbol(symbol) => print_symbol(memory, symbol),
        Instruction::NoOp() => Ok(()),
    }
}

// Write a `String` literal to a symbol.
//
// Can never error, since writing will always succeed.
fn write_string_to_symbol(memory: &mut Memory, symbol: &Symbol, value: &String) -> Result<(), String> {
    memory.write(symbol, Data::new(value));
    Ok(())
}

// Write an `i64` literal to a symbol.
//
// Can never error, since writing will always succeed.
fn write_int_to_symbol(memory: &mut Memory, symbol: &Symbol, value: &i64) -> Result<(), String> {
    memory.write(symbol, Data::new(value));
    Ok(())
}

// Copy the data in `source` to `dest`. Returns an error if `source` is undefined.
fn copy_symbol(memory: &mut Memory, source: &Symbol, dest: &Symbol) -> Result<(), String> {
    memory.copy(source, dest)?;
    Ok(())
}

// Add the integers in `a` and `b` together. Returns an error if either `a` or `b` is undefined, or
// does not contain an integer.
fn add_symbols(memory: &mut Memory, a: &Symbol, b: &Symbol, dest: &Symbol) -> Result<(), String> {
    let a_data = memory.read_typed::<i64>(a)?;
    let b_data = memory.read_typed::<i64>(b)?;
    let result = a_data + b_data;
    memory.write(dest, Data::new(&result));
    Ok(())
}

// Print the data at the symbol to the console. Returns an error if the data is not a printable
// type (currently either a `String` or an `i64`)
fn print_symbol(memory: &Memory, symbol: &Symbol) -> Result<(), String> {
    let data = memory.read_untyped(symbol)?;
    if data.is::<String>() {
        println!("{}", data.downcast_ref::<String>().unwrap()); 
    } else if data.is::<i64>() {
        println!("{}", data.downcast_ref::<i64>().unwrap()); 
    } else {
        log_and_return_err!("Cannot print whatever type is in {}!", symbol.0)
    }
    Ok(())
}
