//! ConcordeVM's Memory system.
//! 
//! Provides linear memory for the VM to use along with utils for reading and writing typed data.

use crate::log_and_return_err;

use log::error;
use std::{cmp, mem};

pub trait ByteSerialisable {
    fn to_bytes(&self) -> Vec<u8>;
    fn from_bytes(bytes: & [u8]) -> Self;
    fn write_bytes_to(&self, vec: &mut Vec<u8>, address: usize);
    fn append_bytes_to(&self, vec: &mut Vec<u8>);
    fn get_size(&self) -> usize;
}

macro_rules! impl_for_numerics {
    ($($t:ty),*) => {
        $(
            impl ByteSerialisable for $t {
                fn to_bytes(&self) -> Vec<u8> {
                    return Vec::from(self.to_ne_bytes());
                }

                fn from_bytes(bytes: & [u8]) -> Self {
                    let buf: [u8; mem::size_of::<Self>()] = bytes.try_into().expect("wrong buffer size for from_bytes");
                    return Self::from_ne_bytes(buf);
                }

                fn write_bytes_to(&self, vec: &mut Vec<u8>, address: usize){
                    for (offset, byte) in self.to_ne_bytes().iter().enumerate() {
                        vec[address + offset] = *byte;
                    }
                }

                fn append_bytes_to(&self, vec: &mut Vec<u8>) {
                    vec.extend(self.to_ne_bytes())
                }

                fn get_size(&self) -> usize {
                    return mem::size_of::<Self>();
                }
            }
        )*
    };
}

impl_for_numerics!(u8, u16, u32, u64, i8, i16, i32, i64, i128, u128, f32, f64);

impl ByteSerialisable for String {
    fn to_bytes(&self) -> Vec<u8> {

        return self.chars().map(|c| c as u8).collect();
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        String::from_utf8(bytes[..end].to_vec()).expect("invalid UTF-8")
    }
    

    fn write_bytes_to(&self, vec: &mut Vec<u8>, address: usize) {
        for (offset, c) in self.chars().enumerate(){
            vec[address + offset] = c as u8;
        }
    }

    fn append_bytes_to(&self, vec: &mut Vec<u8>) {
        vec.extend(self.to_bytes());
    }

    fn get_size(&self) -> usize {
        return self.len();
    }
}

impl ByteSerialisable for bool {
    fn to_bytes(&self) -> Vec<u8> {
        if *self {
            return [1u8].to_vec();
        } else {
            return [0u8].to_vec();
        }
    }

    fn from_bytes(bytes: & [u8]) -> Self {
        return bytes[0] == 1u8;
    }

    fn write_bytes_to(&self, vec: &mut Vec<u8>, address: usize) {
        vec[address] = if *self {1u8} else {0u8};
    }

    fn append_bytes_to(&self, vec: &mut Vec<u8>) {
        vec.extend(self.to_bytes());
    }

    fn get_size(&self) -> usize {
        return 1;
    }
}

impl ByteSerialisable for Vec<u8> {
    fn to_bytes(&self) -> Vec<u8> {
        return self.clone();
    }

    fn from_bytes(bytes: & [u8]) -> Self {
        return bytes.to_vec();
    }

    fn write_bytes_to(&self, vec: &mut Vec<u8>, address: usize) {
        for (offset, value) in self.iter().enumerate() {
            vec[address + offset] = self[address + offset];
        }
    }

    fn append_bytes_to(&self, vec: &mut Vec<u8>) {
        vec.extend(self);
    }

    fn get_size(&self) -> usize {
        return self.len();
    }
}

/// `Memory` is what actually handles reading and writing from the symbol table.
///
/// It wraps a `HashMap<Symbol, Data>` and implements basic memory operations over that, including
/// both typed and untyped reading, writing, and copying.
#[derive(Clone)]
pub struct Memory{
    linear_memory: Vec<u8>,
    write_pointer: usize,
}

impl Memory {
    /// Create a new block of memory
    pub fn new(size: usize) -> Memory {
        return Memory{linear_memory: vec![0; size], write_pointer: 0};
    }

    /// Create a new block of memory with a given capacity
    #[allow(dead_code)]
    pub fn with_capacity(capacity: usize) -> Memory {
        return Memory{linear_memory: Vec::with_capacity(capacity), write_pointer: 0};
    }

    /// Write the given data to the symbol. If the symbol does not already exist, create it.
    ///
    /// Returns nothing and should never be able to fail, since any Symbol can we written to, even
    /// if it is undefined.
    pub fn write(&mut self, address: usize, data: & impl ByteSerialisable) {
        data.write_bytes_to(&mut self.linear_memory, address);
    }

    /// Read from the given symbol, expecting a specific type. Guaranteed to return that type or error.
    ///
    /// If the symbol does not exist, return an error due to trying to read an undefined symbol. If the symbol does exist, but is
    /// not of the expected type, return an error.
    pub fn read_typed<T: ByteSerialisable + 'static>(&self, address: usize) -> T {
        let slice = &self.linear_memory[address..address + mem::size_of::<T>()];
        return T::from_bytes(slice);
    }

    /// Copy the data from source to dest. If dest doesn't exist yet, create it.
    ///
    /// If the source doesn't exist, return an error.
    ///
    /// While this could arguably be implented at the instruction level, having this be a memory
    /// level operation may be good for operations besides just copying.
    pub fn memcpy(&mut self, source: usize, dest: usize, n: usize) -> Result<(), String> {

        if cmp::max(source, dest) + n <= self.linear_memory.capacity() {
            for offset in 0..n {
                self.linear_memory[dest + offset] = self.linear_memory[source + offset];
            };
            return Ok(());
        } else {
            log_and_return_err!("Tried memcpy of {} bytes from {} to {}, but max memory address is {}", n, source, dest, self.linear_memory.capacity());
        }
    }

    /// Get an iterator over all of the symbols currently in memory. Useful for debugging purposes.
    pub fn dump(&self) -> Vec<u8> {
        return self.linear_memory.clone();
    }

    pub fn extend_memory(&mut self, n: usize) {
        self.linear_memory.extend(vec![0u8; n]);
    }
}

impl Default for Memory {
   fn default() -> Self {
       Memory::new(0)
   } 
}
