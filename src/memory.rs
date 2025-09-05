// This module will contain the memory system for ConcordeVM
// We reference memory inside the VM not by address but by symbol.
// Currenly all memory and variables are global, so that this is as simple as possible

use crate::log_and_return_err;

use std::any::type_name;
use std::collections::HashMap;
use log::error;
use cloneable_any::CloneableAny;
use dyn_clone::clone_box;

// Symbols (at least right now) are just String wrappers, so cloning and such is relatively cheap
// for them
#[derive(Hash, Clone, Eq, PartialEq, Debug)]
pub struct Symbol(pub String);

pub struct Data(Box<dyn CloneableAny>);

impl Data {
    // Creating new Data structs should always clone the inner value, since we don't want weird
    // borrows happening. Everything should be owned by the Memory.
    pub fn new<T: Clone + 'static>(value: &T) -> Data {
        Data(Box::new(value.clone()))
    }

    pub fn as_type<T: CloneableAny + 'static>(&self) -> Result<&T, String> {
        match self.0.downcast_ref::<T>() {
            Some(result) => Ok(result),
            None => log_and_return_err!("Could not downcast data to {}!", type_name::<T>())
        }
    }

    pub fn as_ref(&self) -> &dyn CloneableAny {
        self.0.as_ref()
    }

    pub fn clone(&self) -> Data {
        Data(clone_box(self.0.as_ref()))
    }
}

pub struct Memory(HashMap<Symbol, Data>);

impl Memory {
    // Create a new block of memory with a given capacity
    pub fn new() -> Memory {
        Memory(HashMap::new())
    }

    // Create a new block of memory with a given capacity
    #[allow(dead_code)]
    pub fn with_capacity(size: usize) -> Memory {
        Memory(HashMap::with_capacity(size))
    }

    // Write the given data to the symbol
    // If the symbol does not already exist, create it
    pub fn write(&mut self, symbol: &Symbol, data: Data) {
        self.0.insert(symbol.clone(), data);
    }

    // Read from the given symbol, returning an untyped CloneableAny 
    // If the symbol does not exist, return an error
    pub fn read_untyped(&self, symbol: &Symbol) -> Result<&dyn CloneableAny, String> {
        match self.0.get(symbol) {
            Some(data) => Ok(data.as_ref()), 
            None => log_and_return_err!("Tried to read from undefined symbol: {}", symbol.0)
        }
    }

    // Read from the given symbol, attempting to get a specific type
    // If the symbol does not exist, return an error
    pub fn read_typed<T: CloneableAny + 'static>(&self, symbol: &Symbol) -> Result<&T, String> {
        match self.0.get(symbol) {
            Some(data) => {
                let typed_data = data.as_type::<T>()?;
                Ok(typed_data)
            },
            None => log_and_return_err!("Tried to read from undefined symbol: {}", symbol.0)
        }
    }

    // Copy the data from source to dest. If dest doesn't exist yet, create it.
    // If the source doesn't exist, return an error.
    // While it isn't strictly necessary to implement this as part of the memory, it's a purely
    // memory operation, so it kind of makes sense.
    pub fn copy(&mut self, source: &Symbol, dest: &Symbol) -> Result<(), String> {
        match self.0.get(source) {
            Some(data) => {
                self.0.insert(dest.clone(), data.clone());
                Ok(())
            }
            None => log_and_return_err!("Couldn't copy undefined symbol {} to {}!", source.0, dest.0)
        }
    }

}

