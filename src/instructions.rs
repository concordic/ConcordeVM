//! ConcordeVM's instruction set implementation.
//!
//! Provides a function to execute arbitrary instructions as defined by the ConcordeISA. 

use crate::io::ConcordeIO;
use crate::memory::{Data, Memory};
use crate::cpu::ExecutionStack;

use concordeisa::{instructions::Instruction, memory::Symbol};

use log::info;

/// Execute the given instruction and increment the execution pointer.
/// Return an error if something goes wrong. (eg. division by zero, or accessing invalid memory)
//
/// Currently, each instruction from the enum maps to a function of the same name in a `match` statement. There
/// may be a better way to do this that's more extensible. We also handle incrementing the stack
/// only when we need to in the same way, so there's room for improvement.
pub fn execute_instruction(instruction: &Instruction, memory: &mut Memory, io: &mut ConcordeIO, stack: &mut ExecutionStack) -> Result<(), String> {
    info!("Executing instruction {:?}", instruction);
    let result = match instruction {
        // Immediate writes
        Instruction::WriteStringToSymbol(symbol, value) => write_string_to_symbol(memory, symbol, value),
        Instruction::WriteIntToSymbol(symbol, value) => write_int_to_symbol(memory, symbol, value),
        Instruction::WriteBoolToSymbol(symbol, value) => write_bool_to_symbol(memory, symbol, value),
        Instruction::WriteBytesToSymbol(symbol, value) => write_bytes_to_symbol(memory, symbol, value),
        
        // Memory management
        Instruction::CopySymbol(source, dest) => copy_symbol(memory, source, dest),
        
        // Arithmetic
        Instruction::AddSymbols(a, b, dest) => add_symbols(memory, a, b, dest),
        Instruction::SubtractSymbols(a, b, dest) => subtract_symbols(memory, a, b, dest),
        Instruction::CompareEqual(a, b, dest) => compare_equal(memory, a, b, dest),
        Instruction::CompareGreater(a, b, dest) => compare_greater(memory, a, b, dest),
        Instruction::CompareLesser(a, b, dest) => compare_lesser(memory, a, b, dest),
        
        // I/O
        Instruction::OpenStream(name, stream) => open_stream(memory, io, name, stream),
        Instruction::CloseStream(stream) => close_stream(io, stream),
        Instruction::ReadStream(stream, n, dest) => read_stream(memory, io, stream, n, dest),
        Instruction::WriteStream(stream, n, src) => write_stream(memory, io, stream, n, src),
        
        // Flow control
        Instruction::Jump(target) => jump(stack, target),
        Instruction::Goto(target) => goto(stack, target),
        Instruction::JumpIfTrue(target, condition) => jump_if_true(memory, stack, target, condition),
        Instruction::Return() => ret(stack),
        
        // Misc.
        Instruction::NoOp() => Ok(()),

        #[allow(unreachable_patterns)]
        _ => Err("Unimplemented operation!".to_string()),
    };

    // We don't want to increment the stack after jumping, since it'll start execution from the
    // second instruction as a result.
    match instruction {
        Instruction::Jump(_) | Instruction::JumpIfTrue(_, _) => {}
        _ => stack.increment(),
    };

    result
}

/// Write a `String` literal to a symbol.
fn write_string_to_symbol(memory: &mut Memory, symbol: &Symbol, value: &String) -> Result<(), String> {
    memory.write(symbol, Data::new(value));
    Ok(())
}

/// Write an `i64` literal to a symbol.
fn write_int_to_symbol(memory: &mut Memory, symbol: &Symbol, value: &i64) -> Result<(), String> {
    memory.write(symbol, Data::new(value));
    Ok(())
}

/// Write a `bool` literal to a symbol.
fn write_bool_to_symbol(memory: &mut Memory, symbol: &Symbol, value: &bool) -> Result<(), String> {
    memory.write(symbol, Data::new(value));
    Ok(())
}

/// Write a `Vec<u8>` literal to a symbol.
fn write_bytes_to_symbol(memory: &mut Memory, symbol: &Symbol, value: &Vec<u8>) -> Result<(), String> {
    memory.write(symbol, Data::new(value));
    Ok(())
}

/// Copy the data in `source` to `dest`. Returns an error if `source` is undefined.
fn copy_symbol(memory: &mut Memory, source: &Symbol, dest: &Symbol) -> Result<(), String> {
    memory.copy(source, dest)?;
    Ok(())
}

/// Add the integers in `a` and `b` together, and put the result in `dest`.
/// Returns an error if either `a` or `b` is undefined, or does not contain an integer.
fn add_symbols(memory: &mut Memory, a: &Symbol, b: &Symbol, dest: &Symbol) -> Result<(), String> {
    let a_data = memory.read_typed::<i64>(a)?;
    let b_data = memory.read_typed::<i64>(b)?;
    let result = a_data + b_data;
    memory.write(dest, Data::new(&result));
    Ok(())
}

/// Subtract the integer in `b` from `a`, and put the result in `dest`.
/// Returns an error if either `a` or `b` is undefined, or does not contain an integer.
fn subtract_symbols(memory: &mut Memory, a: &Symbol, b: &Symbol, dest: &Symbol) -> Result<(), String> {
    let a_data = memory.read_typed::<i64>(a)?;
    let b_data = memory.read_typed::<i64>(b)?;
    let result = a_data - b_data;
    memory.write(dest, Data::new(&result));
    Ok(())
}

/// Check if the integers in `a` and `b` are equal, and put the result in `dest` 
/// Returns an error if either `a` or `b` is undefined, or does not contain an integer.
fn compare_equal(memory: &mut Memory, a: &Symbol, b: &Symbol, dest: &Symbol) -> Result<(), String> {
    let a_data = memory.read_typed::<i64>(a)?;
    let b_data = memory.read_typed::<i64>(b)?;
    let result = a_data == b_data;
    memory.write(dest, Data::new(&result));
    Ok(())
}

/// Check if the integer in `a` is greater than in `b`, and put the result in `dest` 
/// Returns an error if either `a` or `b` is undefined, or does not contain an integer.
fn compare_greater(memory: &mut Memory, a: &Symbol, b: &Symbol, dest: &Symbol) -> Result<(), String> {
    let a_data = memory.read_typed::<i64>(a)?;
    let b_data = memory.read_typed::<i64>(b)?;
    let result = a_data > b_data;
    memory.write(dest, Data::new(&result));
    Ok(())
}

/// Check if the integer in `a` is lesser than in `b`, and put the result in `dest` 
/// Returns an error if either `a` or `b` is undefined, or does not contain an integer.
fn compare_lesser(memory: &mut Memory, a: &Symbol, b: &Symbol, dest: &Symbol) -> Result<(), String> {
    let a_data = memory.read_typed::<i64>(a)?;
    let b_data = memory.read_typed::<i64>(b)?;
    let result = a_data < b_data;
    memory.write(dest, Data::new(&result));
    Ok(())
}

/// Jump execution to the target symbol. Will not error.
fn jump(stack: &mut ExecutionStack, target: &Symbol) -> Result<(), String> {
    stack.jump(target);
    Ok(())
}
fn goto(stack: &mut ExecutionStack, target: &Symbol) -> Result<(), String> {
    stack.goto(target);
    Ok(())
}

/// Jump execution to the target if the condition is true. Will not error.
fn jump_if_true(memory: &mut Memory, stack: &mut ExecutionStack, target: &Symbol, condition: &Symbol) -> Result<(), String> {
    let c = memory.read_typed::<bool>(condition)?;
    if *c {
        stack.jump(target);
    } else {
        stack.increment();
    }
    Ok(())
}

/// Return execution to the last symbol. Will not error.
fn ret(stack: &mut ExecutionStack) -> Result<(), String> {
    stack.ret();
    Ok(())
}

/// Open a stream in the IO interface.
fn open_stream(memory: &mut Memory, io: &mut ConcordeIO, name: &Symbol, stream: &Symbol) -> Result<(), String> {
    let name_data = memory.read_typed::<String>(name)?;
    io.open(stream, name_data.clone())
}

// Close a stream in the IO interface.
fn close_stream(io: &mut ConcordeIO, stream: &Symbol) -> Result<(), String> {
    io.close(stream)
}

/// Read `n` bytes from `stream` and put the result in `dest`.
fn read_stream(memory: &mut Memory, io: &mut ConcordeIO, stream: &Symbol, n: &Symbol, dest: &Symbol) -> Result<(), String> {
    let n_data = memory.read_typed::<i64>(n)?;
    let (read_data, _read_n) = io.read(stream, usize::try_from(*n_data).unwrap())?;
    memory.write(dest, Data::new(&read_data));
    Ok(())
}

/// Write `n` bytes from `src` into `stream`.
fn write_stream(memory: &mut Memory, io: &mut ConcordeIO, stream: &Symbol, n: &Symbol, src: &Symbol) -> Result<(), String> {
    let write_data = memory.read_typed::<Vec<u8>>(src)?;
    let n_data = memory.read_typed::<i64>(n)?;
    io.write(stream, &write_data[..usize::try_from(*n_data).unwrap()])?;
    Ok(())
}
