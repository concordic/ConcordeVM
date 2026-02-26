//! ConcordeVM's instruction set implementation.
//!
//! Provides a function to execute arbitrary instructions as defined by the ConcordeISA.

use std::future;

use crate::cpu::Program;
use crate::io::ConcordeIO;
use crate::memory::{Memory};

use concordeisa::{instructions::Instruction};

use log::info;

/// Execute the given instruction and increment the execution pointer.
/// Return an error if something goes wrong. (eg. division by zero, or accessing invalid memory)
//
/// Currently, each instruction from the enum maps to a function of the same name in a `match` statement. There
/// may be a better way to do this that's more extensible. We also handle incrementing the stack
/// only when we need to in the same way, so there's room for improvement.
pub fn execute_instruction(
    memory: &mut Memory,
    io: &mut ConcordeIO,
    program: &mut Program,
) -> Result<Interrupt, String> {
    let instruction = program.get_instruction().clone();
    info!("Executing instruction {:?}", instruction);

    let result = match instruction {
        // Immediate writes
        Instruction::WriteStringToSymbol(symbol, ref value) => {
            write_string_to_symbol(memory, symbol, &value)
        }
        Instruction::WriteIntToSymbol(symbol, value) => write_int_to_symbol(memory, symbol, &value),
        Instruction::WriteBoolToSymbol(symbol, value) => {
            write_bool_to_symbol(memory, symbol, &value)
        }
        Instruction::WriteBytesToSymbol(symbol, ref value) => {
            write_bytes_to_symbol(memory, symbol, &value)
        }
        
        // Memory management
        Instruction::MemCpy(source, dest, n) => copy_symbol(memory, source, dest, n),

        // Arithmetic
        Instruction::AddSymbols(a, b, dest) => add_symbols(memory, a, b, dest),
        Instruction::SubtractSymbols(a, b, dest) => subtract_symbols(memory, a, b, dest),
        Instruction::MultiplySymbols(a, b, dest) => multiply_symbols(memory, a, b, dest),
        Instruction::DivideSymbols(a, b, dest) => divide_symbols(memory, a, b, dest),
        Instruction::ModuloSymbols(a, b, dest) => modulo_symbols(memory, a, b, dest),
        Instruction::MinSymbols(a, b, dest) => min_symbols(memory, a, b, dest),
        Instruction::MaxSymbols(a, b, dest) => max_symbols(memory, a, b, dest),
        Instruction::FmaSymbols(a, b, c, dest) => fma_symbols(memory, a, b, c, dest),
        Instruction::SinSymbol(a, dest) => sin_symbol(memory, a, dest),
        Instruction::CosSymbol(a, dest) => cos_symbol(memory, a, dest),
        Instruction::TanSymbol(a, dest) => tan_symbol(memory, a, dest),
        Instruction::ArcsinSymbol(a, dest) => arcsin_symbol(memory, a, dest),
        Instruction::ArccosSymbol(a, dest) => arccos_symbol(memory, a, dest),
        Instruction::ArctanSymbol(a, dest) => arctan_symbol(memory, a, dest),
        Instruction::CompareEqual(a, b, dest) => compare_equal(memory, a, b, dest),
        Instruction::CompareGreater(a, b, dest) => compare_greater(memory, a, b, dest),
        Instruction::CompareLesser(a, b, dest) => compare_lesser(memory, a, b, dest),

        // I/O
        /*
        Instruction::OpenStream(name, stream) => open_stream(memory, io, name, stream),
        Instruction::CloseStream(stream) => close_stream(io, stream),
        Instruction::ReadStream(stream, n, dest) => read_stream(memory, io, stream, n, dest),
        Instruction::WriteStream(stream, n, src) => write_stream(memory, io, stream, n, src),
        */
        
        // Flow control
        Instruction::Jump(target) => jump(program, target),
        Instruction::JumpIfTrue(target, condition) => jump_if_true(memory, program, target, condition),
        Instruction::Await(fut_id, return_write_addr) => Ok(Interrupt::Await(fut_id, return_write_addr)),
        Instruction::CreateCoroutine(dest, arg_addr, n_arg_bytes, write_coro_id_addr) => Ok(Interrupt::CreateCoroutine(dest, arg_addr, n_arg_bytes, write_coro_id_addr)),
        Instruction::Return(address, n) => ret(address, n),
        Instruction::DeleteFuture(future_id) => delete_future(future_id),
        // Misc.
        Instruction::NoOp() => Ok(Interrupt::Ok),

        #[allow(unreachable_patterns)]
        _ => Err("Unimplemented operation!".to_string()),
    };

    // We don't want to increment the stack after jumping, since it'll start execution from the
    // second instruction as a result.
    match instruction {
        Instruction::Jump(_) | Instruction::JumpIfTrue(_, _) => {}
        _ => program.increment(),
    };

    result
}


pub enum Interrupt {
    //    fut id, return write addr
    Await(usize, usize),
    // dest, arg addr, n arg bytes, write coro id addr
    CreateCoroutine(usize, usize, usize, usize),
    //           future id
    DeleteFuture(usize),
    //  ret addr, n_ret_bytes
    Ret(usize, usize),
    Ok,
    EOF
}


fn delete_future(future_id: usize) -> Result<Interrupt, String> {
    return Ok(Interrupt::DeleteFuture(future_id));
}

/// Write a `String` literal to a symbol.
fn write_string_to_symbol(
    memory: &mut Memory,
    address: usize,
    value: &String,
) -> Result<Interrupt, String> {
    memory.write(address, value);
    return Ok(Interrupt::Ok);
}

/// Write an `i64` literal to a symbol.
fn write_int_to_symbol(memory: &mut Memory, address: usize, value: &i64) -> Result<Interrupt, String> {
    memory.write(address, value);
    return Ok(Interrupt::Ok);
}

/// Write a `bool` literal to a symbol.
fn write_bool_to_symbol(memory: &mut Memory, symbol: usize, value: &bool) -> Result<Interrupt, String> {
    memory.write(symbol, value);
    return Ok(Interrupt::Ok);
}

/// Write a `Vec<u8>` literal to a symbol.
fn write_bytes_to_symbol(
    memory: &mut Memory,
    symbol: usize,
    value: &Vec<u8>,
) -> Result<Interrupt, String> {
    memory.write(symbol, value);
    return Ok(Interrupt::Ok);
}

/// Copy the data in `source` to `dest`. Returns an error if `source` is undefined.
fn copy_symbol(memory: &mut Memory, source: usize, dest: usize, n: usize) -> Result<Interrupt, String> {
    memory.memcpy(source, dest, n)?;
    return Ok(Interrupt::Ok);
}

/// Add the integers in `a` and `b` together, and put the result in `dest`.
/// Returns an error if either `a` or `b` is undefined, or does not contain an integer.
fn add_symbols(memory: &mut Memory, a: usize, b: usize, dest: usize) -> Result<Interrupt, String> {
    let a_data = memory.read_typed::<i64>(a);
    let b_data = memory.read_typed::<i64>(b);
    let result = a_data + b_data;
    memory.write(dest, &result);
    return Ok(Interrupt::Ok);
}

/// Subtract the integer in `b` from `a`, and put the result in `dest`.
/// Returns an error if either `a` or `b` is undefined, or does not contain an integer.
fn subtract_symbols(
    memory: &mut Memory,
    a: usize,
    b: usize,
    dest: usize,
) -> Result<Interrupt, String> {
    let a_data = memory.read_typed::<i64>(a);
    let b_data = memory.read_typed::<i64>(b);
    let result = a_data - b_data;
    memory.write(dest, &result);
    return Ok(Interrupt::Ok);
}

/// Multiply the integers in `a` and `b`, and put the result in `dest`.
/// Returns an error if either `a` or `b` is undefined, or does not contain an integer.
fn multiply_symbols(
    memory: &mut Memory,
    a: usize,
    b: usize,
    dest: usize,
) -> Result<Interrupt, String> {
    let a_data = memory.read_typed::<i64>(a);
    let b_data = memory.read_typed::<i64>(b);
    let result = a_data * b_data;
    memory.write(dest, &result);
    return Ok(Interrupt::Ok);
}

/// Divide the integer in `a` by `b`, and put the result in `dest`.
/// Returns an error if either `a` or `b` is undefined, does not contain an integer, or if `b` is zero.
fn divide_symbols(
    memory: &mut Memory,
    a: usize,
    b: usize,
    dest: usize,
) -> Result<Interrupt, String> {
    let a_data = memory.read_typed::<i64>(a);
    let b_data = memory.read_typed::<i64>(b);
    if b_data == 0 {
        return Err("Division by zero error".to_string());
    }
    let result = a_data / b_data;
    memory.write(dest, &result);
    return Ok(Interrupt::Ok);
}

/// Modulo the integer in `a` by `b`, and put the result in `dest`.
/// Returns an error if either `a` or `b` is undefined, does not contain an integer, or if `b` is zero.
fn modulo_symbols(
    memory: &mut Memory,
    a: usize,
    b: usize,
    dest: usize,
) -> Result<Interrupt, String> {
    let a_data = memory.read_typed::<i64>(a);
    let b_data = memory.read_typed::<i64>(b);
    if b_data == 0 {
        return Err("Division by zero error".to_string());
    }
    let result = a_data % b_data;
    memory.write(dest, &result);
    return Ok(Interrupt::Ok);
}

/// Perform fused multiply-add on the integers in `a`, `b`, and `c` (a * b + c), and put the result in `dest`.
/// Returns an error if any of `a`, `b`, or `c` is undefined, or does not contain an integer.
fn fma_symbols(
    memory: &mut Memory,
    a: usize,
    b: usize,
    c: usize,
    dest: usize,
) -> Result<Interrupt, String> {
    let a_data = memory.read_typed::<i64>(a);
    let b_data = memory.read_typed::<i64>(b);
    let c_data = memory.read_typed::<i64>(c);
    let result = a_data * b_data + c_data;
    memory.write(dest, &result);
    return Ok(Interrupt::Ok);
}

/// Calculate the minimum of the integers in `a` and `b`, and put the result in `dest`.
/// Returns an error if `a` or `b` is undefined, or does not contain an integer.
fn min_symbols(memory: &mut Memory, a: usize, b: usize, dest: usize) -> Result<Interrupt, String> {
    let a_data = memory.read_typed::<i64>(a);
    let b_data = memory.read_typed::<i64>(b);
    let result = std::cmp::min(a_data, b_data);
    memory.write(dest, &result);
    return Ok(Interrupt::Ok);
}

/// Calculate the maximum of the integers in `a` and `b`, and put the result in `dest`.
/// Returns an error if `a` or `b` is undefined, or does not contain an integer.
fn max_symbols(memory: &mut Memory, a: usize, b: usize, dest: usize) -> Result<Interrupt, String> {
    let a_data = memory.read_typed::<i64>(a);
    let b_data = memory.read_typed::<i64>(b);
    let result = std::cmp::max(a_data, b_data);
    memory.write(dest, &result);
    return Ok(Interrupt::Ok);
}
/// Calculate the sine of the float in `a`, and put the result in `dest`.
/// Returns an error if `a` is undefined, or does not contain a float.
fn sin_symbol(memory: &mut Memory, a: usize, dest: usize) -> Result<Interrupt, String> {
    let a_data = memory.read_typed::<f64>(a);
    let result = a_data.sin();
    memory.write(dest, &result);
    return Ok(Interrupt::Ok);
}

/// Calculate the cosine of the float in `a`, and put the result in `dest`.
/// Returns an error if `a` is undefined, or does not contain a float.
fn cos_symbol(memory: &mut Memory, a: usize, dest: usize) -> Result<Interrupt, String> {
    let a_data = memory.read_typed::<f64>(a);
    let result = a_data.cos();
    memory.write(dest, &result);
    return Ok(Interrupt::Ok);
}

/// Calculate the tangent of the float in `a`, and put the result in `dest`.
/// Returns an error if `a` is undefined, or does not contain a float.
fn tan_symbol(memory: &mut Memory, a: usize, dest: usize) -> Result<Interrupt, String> {
    let a_data = memory.read_typed::<f64>(a);
    let result = a_data.tan();
    memory.write(dest, &result);
    return Ok(Interrupt::Ok);
}

/// Calculate the arcsine of the float in `a`, and put the result in `dest`.
/// Returns an error if `a` is undefined, or does not contain a float.
fn arcsin_symbol(memory: &mut Memory, a: usize, dest: usize) -> Result<Interrupt, String> {
    let a_data = memory.read_typed::<f64>(a);
    let result = a_data.asin();
    memory.write(dest, &result);
    return Ok(Interrupt::Ok);
}

/// Calculate the arccosine of the float in `a`, and put the result in `dest`.
/// Returns an error if `a` is undefined, or does not contain a float.
fn arccos_symbol(memory: &mut Memory, a: usize, dest: usize) -> Result<Interrupt, String> {
    let a_data = memory.read_typed::<f64>(a);
    let result = a_data.acos();
    memory.write(dest, &result);
    return Ok(Interrupt::Ok);
}

/// Calculate the arctangent of the float in `a`, and put the result in `dest`.
/// Returns an error if `a` is undefined, or does not contain a float.
fn arctan_symbol(memory: &mut Memory, a: usize, dest: usize) -> Result<Interrupt, String> {
    let a_data = memory.read_typed::<f64>(a);
    let result = a_data.atan();
    memory.write(dest, &result);
    return Ok(Interrupt::Ok);
}

/// Check if the integers in `a` and `b` are equal, and put the result in `dest`
/// Returns an error if either `a` or `b` is undefined, or does not contain an integer.
fn compare_equal(memory: &mut Memory, a: usize, b: usize, dest: usize) -> Result<Interrupt, String> {
    let a_data = memory.read_typed::<i64>(a);
    let b_data = memory.read_typed::<i64>(b);
    let result = a_data == b_data;
    memory.write(dest, &result);
    return Ok(Interrupt::Ok);
}

/// Check if the integer in `a` is greater than in `b`, and put the result in `dest`
/// Returns an error if either `a` or `b` is undefined, or does not contain an integer.
fn compare_greater(
    memory: &mut Memory,
    a: usize,
    b: usize,
    dest: usize,
) -> Result<Interrupt, String> {
    let a_data = memory.read_typed::<i64>(a);
    let b_data = memory.read_typed::<i64>(b);
    let result = a_data > b_data;
    memory.write(dest, &result);
    return Ok(Interrupt::Ok);
}

/// Check if the integer in `a` is lesser than in `b`, and put the result in `dest`
/// Returns an error if either `a` or `b` is undefined, or does not contain an integer.
fn compare_lesser(
    memory: &mut Memory,
    a: usize,
    b: usize,
    dest: usize,
) -> Result<Interrupt, String> {
    let a_data = memory.read_typed::<i64>(a);
    let b_data = memory.read_typed::<i64>(b);
    let result = a_data < b_data;
    memory.write(dest, &result);
    return Ok(Interrupt::Ok);
}

/// Jump execution to the target symbol. Will not error.
fn jump(stack: &mut Program, target: usize) -> Result<Interrupt, String> {
    stack.jump(target);
    return Ok(Interrupt::Ok);
}

/// Jump execution to the target if the condition is true. Will not error.
fn jump_if_true(
    memory: &mut Memory,
    stack: &mut Program,
    target: usize,
    condition: usize,
) -> Result<Interrupt, String> {
    let c = memory.read_typed::<bool>(condition);
    if c {
        stack.jump(target);
    } else {
        stack.increment();
    }
    return Ok(Interrupt::Ok);
}

/// Return execution to the last symbol. Will not error.
fn ret(address: usize, n: usize) -> Result<Interrupt, String> {
    return Ok(Interrupt::Ret(address, n));
}

fn extend_memory(memory: &mut Memory, n: usize) -> Result<Interrupt, String> {
    memory.extend_memory(n);
    return Ok(Interrupt::Ok);
}

/// Open a stream in the IO interface.
fn open_stream(
    memory: &mut Memory,
    io: &mut ConcordeIO,
    name: usize,
    stream: usize,
) -> Result<Interrupt, String> {
    let name_data: String = memory.read_typed::<String>(name);
    io.open(&stream, name_data.clone())?;
    return Ok(Interrupt::Ok);
}

// Close a stream in the IO interface.
fn close_stream(io: &mut ConcordeIO, stream: usize) -> Result<Interrupt, String> {
    io.close(&stream)?;
    return Ok(Interrupt::Ok);
}

/// Read `n` bytes from `stream` and put the result in `dest`.
fn read_stream(
    memory: &mut Memory,
    io: &mut ConcordeIO,
    stream: usize,
    n: usize,
    dest: usize,
) -> Result<Interrupt, String> {
    let n_data = memory.read_typed::<i64>(n);
    let (read_data, _read_n) = io.read(&stream, usize::try_from(n_data).unwrap())?;
    memory.write(dest, &read_data);
    return Ok(Interrupt::Ok);
}

/// Write `n` bytes from `src` into `stream`.
fn write_stream(
    memory: &mut Memory,
    io: &mut ConcordeIO,
    stream: usize,
    n: usize,
    src: usize,
) -> Result<Interrupt, String> {
    let write_data = memory.read_typed::<Vec<u8>>(src);
    let n_data = memory.read_typed::<i64>(n);
    io.write(&stream, &write_data[..usize::try_from(n_data).unwrap()])?;
    return Ok(Interrupt::Ok);
}