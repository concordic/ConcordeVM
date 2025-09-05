// This module will contain the memory system for ConcordeVM
// We reference memory inside the VM not by address but by symbol.
// Currenly all memory and variables are global, so that this is as simple as possible

use crate::errors::log_and_return_err;

use std::any::{Any, type_name};
use std::collections::HashMap;
use dyn_clone::{DynClone, clone_box};

#[derive(Hash, Clone, Eq, PartialEq, Debug)]
pub struct Symbol(pub String);

pub struct Data(pub Box<dyn DynClone>);

impl Data {
    pub fn as_type<T: 'static + DynClone>(&self) -> Result<&T, String> {
        match self.0.downcast_ref::<T>() {
            Some(result) => Ok(result),
            None => log_and_return_err!("Could not downcast data to {}!", type_name::<T>())
        }
    }

    pub fn clone(&self) -> Data {
        Data(clone_box(&*self.0))
    }
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

    // Read from the given symbol
    // If the symbol does not exist, return an error
    pub fn read(&self, symbol: &Symbol) -> Result<&Data, String> {
        let data = self.0.get(symbol);
        match data {
            Some(good_data) => Ok(good_data),
            None => log_and_return_err!("Tried to read from an undefined symbol!")
        }
    }
}

