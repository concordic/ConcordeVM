// This module contains the instruction set and how to execute each one
// The instruction set should be the bare minimum to allow ConcordeVM to interact with the OS and
// memory
// No instruction here should be able to be defined as a composition of other instructions (unless
// theres a huge speed penalty or something, then maybe)

use crate::memory::*;

use log::{info, warn};
use std::any::Any;

#[derive(Clone)]
pub enum Instruction {
    WriteStringToSymbol(Symbol, String),
    WriteUSizeToSymbol(Symbol, usize),
    CopySymbol(Symbol, Symbol),
    PrintSymbol(Symbol),
    NoOp(),
}

pub fn execute_instruction(instruction: &Instruction, memory: &mut Memory) {
    match instruction {
        Instruction::WriteStringToSymbol(symbol, value) => write_string_to_symbol(memory, symbol, value),
        Instruction::WriteUSizeToSymbol(symbol, value) => write_usize_to_symbol(memory, symbol, value),
        Instruction::CopySymbol(symbol_source, symbol_dest) => copy_symbol(memory, symbol_source, symbol_dest),
        Instruction::PrintSymbol(symbol) => print_symbol(memory, symbol),
        Instruction::NoOp() => {},
    }
}

fn write_string_to_symbol(memory: &mut Memory, symbol: &Symbol, value: &String) {
    let data = Data(Box::new(value.clone()));
    memory.write(symbol.clone(), data);
}

fn write_usize_to_symbol(memory: &mut Memory, symbol: &Symbol, value: &usize) {
    let data = Data(Box::new(value.clone()));
    memory.write(symbol.clone(), data);
}

fn copy_symbol(memory: &mut Memory, symbol_source: &Symbol, symbol_dest: &Symbol) {
    match memory.read(symbol_source) {
        Ok(val) => memory.write(symbol_dest.clone(), val.clone()),
        Err(e) => panic!("{}", e)
    }
}

fn print_symbol(memory: &Memory, symbol: &Symbol) {
    
}
