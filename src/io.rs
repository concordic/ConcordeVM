//! ConcordeVM's IO System.
//!
//! Currently only supports opening files.

use crate::log_and_return_err;

use std::fs::{rename, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::collections::HashMap;
use concordeisa::memory::Symbol;
use log::error;
use io_streams::*;

/// Stream object for Concorde to interface with system IO.
pub struct ConcordeStream {
    name: String,
    // Replace with BufDuplexer
    reader: BufReader<StreamReader>,
    writer: BufWriter<StreamWriter>,
    has_written: bool,
}

impl ConcordeStream {
    pub fn open(name: &String) -> Result<ConcordeStream, String> {
        // "stdio" is a reserved name for stdin/stdout
        if name == "stdio" {
            return Ok(ConcordeStream {
                name: name.clone(),
                reader: BufReader::new(StreamReader::stdin().unwrap()),
                writer: BufWriter::new(StreamWriter::stdout().unwrap()),
                has_written: false,
            })
        }
        let file_read = File::options().read(true).write(false).open(&name);
        let mut out_name = name.clone();
        out_name.push_str(".tmp");
        let file_write = File::options().read(false).write(true).open(&out_name);
        match (file_read, file_write) {
            (Ok(fr), Ok(fw)) => {
                Ok(ConcordeStream {
                    name: name.clone(),
                    reader: BufReader::new(StreamReader::file(fr)),
                    writer: BufWriter::new(StreamWriter::file(fw)),
                    has_written: false,
                })
            },
            _ => log_and_return_err!("Could not open file {}!", &name)
        }
    }

    pub fn read(&mut self, n: usize) -> Result<(Vec<u8>, usize), String> {
        let mut buf: Vec<u8> = vec![0; n];
        match self.reader.read(&mut buf[..]) {
            Ok(n) => Ok((buf, n)),
            Err(e) => log_and_return_err!("Failed to read from {}: {}", self.name, e),
        }
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<usize, String>{
        match self.writer.write(buf) {
            Ok(n) => {
                self.has_written =  true;
                Ok(n)
            }
            Err(e) => log_and_return_err!("Failed to write to {}: {}", self.name, e)
        }
    }

    pub fn close(self) -> Result<(), String> {
        drop(self.reader);
        drop(self.writer);
        if self.name != "stdio" {
            let mut out_name = self.name.clone();
            out_name.push_str(".tmp");
            match rename(out_name, self.name.clone()) {
                Ok(()) => return Ok(()),
                Err(e) => log_and_return_err!("Failed to close file {}: {}", self.name, e)
            }
        }
        Ok(())
    }
}

pub struct ConcordeIO(HashMap<Symbol, ConcordeStream>);

impl ConcordeIO {
    pub fn new() -> ConcordeIO {
        ConcordeIO(HashMap::new())
    }

    pub fn open(&mut self, name: &Symbol) -> Result<(), String> {
        let stream = ConcordeStream::open(&name.0);
        if stream.is_err() {
            log_and_return_err!("{}", stream.err().unwrap());
        }
        self.0.insert(name.clone(), stream.ok().unwrap());
        Ok(())
    }

    pub fn read(&mut self, name: &Symbol, n: usize) -> Result<(Vec<u8>, usize), String> {
        match self.0.get_mut(name) {
            Some(stream) => stream.read(n),
            None => log_and_return_err!("Tried to read from undefined stream {}", name.0),
        }
    }

    pub fn write(&mut self, name: &Symbol, buf: &[u8]) -> Result<usize, String> {
        match self.0.get_mut(name) {
            Some(stream) => stream.write(buf),
            None => log_and_return_err!("Tried to write to undefined stream {}", name.0),
        }
    }

    pub fn close(&mut self, name: &Symbol) -> Result<(), String> {
        match self.0.remove(name) {
            Some(stream) => stream.close(),
            None => log_and_return_err!("Tried to close undefined stream {}", name.0),
        }
    }
}
