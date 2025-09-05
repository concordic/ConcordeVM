// This module contains the instruction set and how to execute each one
// The instruction set should be the bare minimum to allow ConcordeVM to interact with the OS and
// memory
// No instruction here should be able to be defined as a composition of other instructions (unless
// theres a huge speed penalty or something, then maybe)

use crate::memory::*;

#[derive(Clone)]
pub enum Instruction {
    // Immediate writes
    WriteStringToSymbol(Symbol, String),
    WriteIntToSymbol(Symbol, i64),

    // Memory management
    // CopySymbol(Symbol, Symbol),
    
    // Arithmetic
    AddSymbols(Symbol, Symbol, Symbol),

    // I/O
    PrintSymbol(Symbol),

    // Misc.
    NoOp(),
}

pub fn execute_instruction(instruction: &Instruction, memory: &mut Memory) -> Result<(), String> {
    match instruction {
        Instruction::WriteStringToSymbol(symbol, value) => write_string_to_symbol(memory, symbol, value),
        Instruction::WriteIntToSymbol(symbol, value) => write_usize_to_symbol(memory, symbol, value),
        // Instruction::CopySymbol(source, dest) => copy_symbol(memory, symbol_source, symbol_dest),
        Instruction::AddSymbols(a, b, dest) => add_symbols(memory, a, b, dest),
        Instruction::PrintSymbol(symbol) => print_symbol(memory, symbol),
        Instruction::NoOp() => Ok(()),
    }
}

fn write_string_to_symbol(memory: &mut Memory, symbol: &Symbol, value: &String) -> Result<(), String> {
    let data = Data(Box::new(value.clone()));
    memory.write(symbol, data);
    Ok(())
}

fn write_usize_to_symbol(memory: &mut Memory, symbol: &Symbol, value: &i64) -> Result<(), String> {
    let data = Data(Box::new(value.clone()));
    memory.write(symbol, data);
    Ok(())
}

fn copy_symbol(memory: &mut Memory, source: &Symbol, dest: &Symbol) -> Result<(), String> {
    memory.copy(source, dest)?;
    Ok(())
}

// Errors if either a or b are not integers
fn add_symbols(memory: &mut Memory, a: &Symbol, b: &Symbol, dest: &Symbol) -> Result<(), String> {
    let a_data = memory.read::<i64>(a)?;
    let b_data = memory.read::<i64>(b)?;
    let result = Data(Box::new(a_data + b_data));
    memory.write(dest, result);
    Ok(())
}

fn print_symbol(memory: &Memory, symbol: &Symbol) -> Result<(), String> {
    Ok(())
}
