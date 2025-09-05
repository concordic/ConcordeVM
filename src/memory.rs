// This module will contain the memory system for ConcordeVM
// We reference memory inside the VM not by address but by symbol.
// Currenly all memory and variables are global, so that this is as simple as possible

use crate::errors::log_and_return_err;

use std::any::{Any, type_name};
use std::collections::HashMap;

#[derive(Hash, Clone, Eq, PartialEq, Debug)]
pub struct Symbol(pub String);

pub struct Data(pub Box<dyn Any>);

impl Data {
    pub fn as_type<T: 'static>(&self) -> Result<&T, String> {
        match self.0.downcast_ref::<T>() {
            Some(result) => Ok(result),
            None => log_and_return_err!("Could not downcast data to {}!", type_name::<T>())
        }
    }

    // pub fn copy_into(&self, dest: &Data) {
    //     let ptr = Box::into_raw(&self.0);
    //     unsafe {
    //         self.0 = Box::from_raw(ptr);
    //     };
    // }
}

pub struct Memory(HashMap<Symbol, Data>);

impl Memory {
    // Create a new block of memory with a given capacity
    pub fn new() -> Memory {
        Memory(HashMap::new())
    }

    // Create a new block of memory with a given capacity
    pub fn with_capacity(size: usize) -> Memory {
        Memory(HashMap::with_capacity(size))
    }

    // Write the given data to the symbol
    // If the symbol does not already exist, create it
    pub fn write(&mut self, symbol: Symbol, data: Data) {
        self.0.insert(symbol, data);
    }

    // Read from the given symbol, attempting to get a specific type
    // If the symbol does not exist, return an error
    pub fn read<T: 'static>(&self, symbol: &Symbol) -> Result<&T, String> {
        match self.0.get(symbol) {
            Some(good_data) => {
                match good_data.as_type::<T>() {
                    Ok(typed_data) => Ok(typed_data),
                    Err(e) => log_and_return_err!("Couldn't read symbol {} as type {}:\n\t => {}", symbol.0, type_name::<T>(), e)
                }
            },
            None => log_and_return_err!("Tried to read from undefined symbol: {}", symbol.0)
        }
    }
}

