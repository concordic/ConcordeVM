//! ConcordeVM's Memory system.
//! 
//! Provides a symbol table that acts as ConcordeVM's "RAM"
//! 
//! We chose to use a symbol table because it's an inherently safer form of memory, especially
//! once we implement proper scoping for it, since you cannot access memory without having access
//! to the symbol you need.

use crate::log_and_return_err;

use concordeisa::{memory::Symbol};

use std::any::type_name;
use std::collections::HashMap;
use log::error;
use cloneable_any::CloneableAny;
use dyn_clone::clone_box;

// `Data` is a wrapper struct for the actual data stored in memory.
//
// It wraps a `Box<dyn CloneableAny>`, which allows any cloneable type to be stored in it, on the
// heap. Data stored in memory must be cloneable, as we need to be able to clone it to get owned
// copies when performing certain operations.
pub struct Data(Box<dyn CloneableAny>);

impl Data {
    // Create a new `Data` struct containing a clone of the given value.
    // 
    // We always clone when creating new Data, since we want to have ownership over the contents,
    // and because the lifetime of the passed value is not guaranteed to last as long as we want to.
    pub fn new<T: Clone + 'static>(value: &T) -> Data {
        Data(Box::new(value.clone()))
    }

    // Downcast the data to a specific type. If the data is the wrong type, returns an error.
    //
    // The type must implement `Clone`, for reasons described above.
    pub fn as_type<T: CloneableAny + 'static>(&self) -> Result<&T, String> {
        match self.0.downcast_ref::<T>() {
            Some(result) => Ok(result),
            None => log_and_return_err!("Could not downcast data to {}!", type_name::<T>())
        }
    }
}

impl AsRef<dyn CloneableAny> for Data {
    fn as_ref(&self) -> &dyn CloneableAny {
        self.0.as_ref()
    }
}

impl Clone for Data {
    fn clone(&self) -> Data {
        Data(clone_box(self.as_ref()))
    }
}

// `Memory` is what actually handles reading and writing from the symbol table.
//
// It wraps a `HashMap<Symbol, Data>` and implements basic memory operations over that, including
// both typed and untyped reading, writing, and copying.
pub struct Memory(HashMap<Symbol, Data>);

impl Memory {
    // Create a new block of memory
    pub fn new() -> Memory {
        Memory(HashMap::new())
    }

    // Create a new block of memory with a given capacity
    #[allow(dead_code)]
    pub fn with_capacity(size: usize) -> Memory {
        Memory(HashMap::with_capacity(size))
    }

    // Write the given data to the symbol. If the symbol does not already exist, create it.
    //
    // Returns nothing and should never be able to fail, since any Symbol can we written to, even
    // if it is undefined.
    pub fn write(&mut self, symbol: &Symbol, data: Data) {
        self.0.insert(symbol.clone(), data);
    }

    // Read from the given symbol, returning an untyped `CloneableAny`.
    //
    // If the symbol does not exist, return an error due to trying to read an undefined symbol. 
    pub fn read_untyped(&self, symbol: &Symbol) -> Result<&dyn CloneableAny, String> {
        match self.0.get(symbol) {
            Some(data) => Ok(data.as_ref()), 
            None => log_and_return_err!("Tried to read from undefined symbol: {}", symbol.0)
        }
    }

    // Read from the given symbol, expecting a specific type. Guaranteed to return that type or error.
    //
    // If the symbol does not exist, return an error due to trying to read an undefined symbol. If the symbol does exist, but is
    // not of the expected type, return an error.
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
    //
    // If the source doesn't exist, return an error.
    //
    // While this could arguably be implented at the instruction level, having this be a memory
    // level operation may be good for operations besides just copying.
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

impl Default for Memory {
   fn default() -> Self {
       Memory::new()
   } 
}
