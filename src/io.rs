//! ConcordeVM's IO System.
//!
//! Currently only supports opening files.

use crate::log_and_return_err;

use std::fs::{rename, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::collections::HashMap;
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
    /// Open a new stream
    ///
    /// Using the name "stdio" will open the standard in/out stream.
    /// Other names will be interpreted as files.
    /// File IO is handled as such:
    ///   - The file on disk with the given name is opened for reading.
    ///   - [filename].tmp is opened for writing.
    ///   - If anything is ever written to the file, a flag is set.
    ///   - When closing the file, if the above flag is set, [filename].tmp gets renamed to [filename].
    pub fn open(name: &String) -> Result<ConcordeStream, String> {
        if name == "stdio" {
            return Ok(ConcordeStream {
                name: name.clone(),
                reader: BufReader::new(StreamReader::stdin().unwrap()),
                writer: BufWriter::new(StreamWriter::stdout().unwrap()),
                has_written: false,
            })
        }
        let file_read = File::options().read(true).write(false).open(name);
        let mut out_name = name.clone();
        out_name.push_str(".tmp");
        let file_write = File::options().read(false).write(true).open(out_name);
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

    /// Attempt to read up to n bytes from the stream.
    /// Returns the read data, as well as the number of bytes read.
    pub fn read(&mut self, n: usize) -> Result<(Vec<u8>, usize), String> {
        let mut buf: Vec<u8> = vec![0; n];
        match self.reader.read(&mut buf[..]) {
            Ok(n) => Ok((buf, n)),
            Err(e) => log_and_return_err!("Failed to read from {}: {}", self.name, e),
        }
    }

    /// Attempt to write all the contents of buf to the stream.
    /// Returns the number of bytes written if successful.
    pub fn write(&mut self, buf: &[u8]) -> Result<usize, String>{
        match self.writer.write(buf) {
            Ok(n) => {
                self.has_written =  true;
                Ok(n)
            }
            Err(e) => log_and_return_err!("Failed to write to {}: {}", self.name, e)
        }
    }

    /// Close the stream.
    /// Copies the temporary file to replace the existing one if anything was written to it.
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

/// Concorde's IO interface. This is what the CPU uses to make IO calls.
pub struct ConcordeIO(HashMap<usize, ConcordeStream>);

impl ConcordeIO {
    // Make a new empty IO interface.
    pub fn new() -> ConcordeIO {
        ConcordeIO(HashMap::new())
    }

    /// Open `filename` under the symbol `name`.
    pub fn open(&mut self, name: &usize, filename: String) -> Result<(), String> {
        let stream = ConcordeStream::open(&filename);
        if stream.is_err() {
            log_and_return_err!("{}", stream.err().unwrap());
        }
        self.0.insert(name.clone(), stream.ok().unwrap());
        Ok(())
    }

    /// Read `n` bytes from the stream at the given symbol.
    /// Returns the read data and the number of bytes read.
    pub fn read(&mut self, name: &usize, n: usize) -> Result<(Vec<u8>, usize), String> {
        match self.0.get_mut(name) {
            Some(stream) => stream.read(n),
            None => log_and_return_err!("Tried to read from undefined stream {}", name),
        }
    }

    /// Write the contents of `buf` to the stream at the given symbol.
    /// Returns the number of bytes written.
    pub fn write(&mut self, name: &usize, buf: &[u8]) -> Result<usize, String> {
        match self.0.get_mut(name) {
            Some(stream) => stream.write(buf),
            None => log_and_return_err!("Tried to write to undefined stream {}", name),
        }
    }

    /// Close the given stream.
    pub fn close(&mut self, name: &usize) -> Result<(), String> {
        match self.0.remove(name) {
            Some(stream) => stream.close(),
            None => log_and_return_err!("Tried to close undefined stream {}", name),
        }
    }
}
