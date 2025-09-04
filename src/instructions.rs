// This module contains the instruction set and how to execute each one
// The instruction set should be the bare minimum to allow ConcordeVM to interact with the OS and
// memory
// No instruction here should be able to be defined as a composition of other instructions (unless
// theres a huge speed penalty or something, then maybe)

use crate::memory::*;

use std::any::Any;

pub enum Instruction {
    WriteConstantToSymbol(Symbol, &dyn Any),
    PrintSymbol(Symbol),
}

pub fn executeInstruction(instruction: Instruction, memory: Memory) {
    match instruction {
        Instruction::WriteConstantToSymbol(symbol, value) => writeConstantToSymbol(memory, symbol, value),
        Instruction::PrintSymbol(symbol) => printSymbol(memory, symbol),
    }
}

fn writeConstantToSymbol(memory: Memory, symbol: Symbol, value: Data) {

}

fn printSymbol(memory: Memory, symbol: Symbol) {

}
