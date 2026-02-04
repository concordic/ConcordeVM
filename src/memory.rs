//! ConcordeVM's Memory system.
// Each instance of Memory is dedicated to a single function call
// Parameters are loaded into memory somehow
// references across memory blocks (ie. borrows) work. somehow
// returned values are put back into memory somehow


use crate::log_and_return_err;

use concordeisa::{memory::Symbol};

use log::error;
use std::vec::Vec;

pub struct Memory(Vec<u8>);

impl Memory {
    pub fn new() -> Self {
        Memory(Vec::new())
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn read(addr: usize) {

    }
}
