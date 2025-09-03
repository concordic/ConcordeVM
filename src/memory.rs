// This module will contain the memory system for ConcordeVM
// We reference memory inside the VM not by address but by symbol.
// Currenly all memory and variables are global, so that this is as simple as possible

use std::any::Any;
use std::collections::HashMap;

#[derive(Hash, Eq, PartialEq, Debug)]
pub struct Symbol(String);

pub struct Data(Box<dyn Any>);

pub struct Memory(HashMap<Symbol, Data>);

impl Memory {
    fn new(size: usize) -> Memory {
        Memory(HashMap::with_capacity(size))
    }

    // Write the given data to the symbol
    // If the symbol does not already exist, create it
    fn write(&mut self, symbol: Symbol, data: Data) {
        self.0.insert(symbol, data);
    }

    // Read from the given symbol
    // If the symbol does not exist, return an error
    fn read(&self, symbol: Symbol) -> Result<&Data, &'static str> {
        let data = self.0.get(&symbol);
        match data {
            Some(good_data) => Ok(good_data),
            None => Err("Tried to read undefined value!"),
        }
    }
}

