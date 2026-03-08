//! ConcordeVM's instruction set implementation.
//!
//! Provides a function to execute arbitrary instructions as defined by the ConcordeISA.

use crate::cpu::Program;
use crate::io::ConcordeIO;
use crate::memory::{ByteParseable, ByteSerialisable, Memory};
use libffi::middle::Type;

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
        Instruction::WriteStringToSymbol(symbol, ref value) => write_to_symbol::<String>(memory, symbol, &value),
        Instruction::WriteIntToSymbol(symbol, value) => write_to_symbol::<i64>(memory, symbol, &value),
        Instruction::WriteBoolToSymbol(symbol, value) => write_to_symbol::<bool>(memory, symbol, &value),
        Instruction::WriteBytesToSymbol(symbol, ref value) => write_to_symbol::<Vec<u8>>(memory, symbol, value),

        // Memory management
        Instruction::MemCpy(source, dest, n) => copy_symbol(memory, source, dest, n),
        Instruction::MemExtend(n_bytes) => extend_memory(memory, n_bytes),
        Instruction::MemExtendTo(n_bytes)=> extend_memory_to(memory, n_bytes),
        Instruction::Ind(addr_location, dest, n) => ind(memory, addr_location, dest, n),

        // Arithmetic (force integral ops to i64)
        Instruction::AddSymbols(a, b, dest) => add_symbols::<i64>(memory, a, b, dest),
        Instruction::SubtractSymbols(a, b, dest) => subtract_symbols::<i64>(memory, a, b, dest),
        Instruction::MultiplySymbols(a, b, dest) => multiply_symbols::<i64>(memory, a, b, dest),
        Instruction::DivideSymbols(a, b, dest) => divide_symbols::<i64>(memory, a, b, dest),
        Instruction::ModuloSymbols(a, b, dest) => modulo_symbols::<i64>(memory, a, b, dest),
        Instruction::MinSymbols(a, b, dest) => min_symbols::<i64>(memory, a, b, dest),
        Instruction::MaxSymbols(a, b, dest) => max_symbols::<i64>(memory, a, b, dest),
        Instruction::FmaSymbols(a, b, c, dest) => fma_symbols::<i64>(memory, a, b, c, dest),

        // Trig (force to f32)
        Instruction::SinSymbol(a, dest) => sin_symbol::<f32>(memory, a, dest),
        Instruction::CosSymbol(a, dest) => cos_symbol::<f32>(memory, a, dest),
        Instruction::TanSymbol(a, dest) => tan_symbol::<f32>(memory, a, dest),
        Instruction::ArcsinSymbol(a, dest) => arcsin_symbol::<f32>(memory, a, dest),
        Instruction::ArccosSymbol(a, dest) => arccos_symbol::<f32>(memory, a, dest),
        Instruction::ArctanSymbol(a, dest) => arctan_symbol::<f32>(memory, a, dest),

        // Comparisons (also integral -> i64)
        Instruction::CompareEqual(a, b, dest) => compare_equal::<i64>(memory, a, b, dest),
        Instruction::CompareGreater(a, b, dest) => compare_greater::<i64>(memory, a, b, dest),
        Instruction::CompareLesser(a, b, dest) => compare_lesser::<i64>(memory, a, b, dest),

        // Flow control
        Instruction::Jump(target) => jump(program, target),
        Instruction::JumpIfTrue(target, condition) => jump_if_true(memory, program, target, condition),
        Instruction::Await(fut_id_location, return_write_addr) => Ok(Interrupt::Await(memory.read_typed::<usize>(fut_id_location), return_write_addr)),
        Instruction::CreateCoroutine(dest, arg_addr, n_arg_bytes, write_coro_id_addr) => Ok(Interrupt::CreateCoroutine(dest, arg_addr, n_arg_bytes, write_coro_id_addr)),
        Instruction::Return(address, n) => ret(address, n),
        Instruction::DeleteFuture(future_id) => delete_future(future_id),

        
        Instruction::LoadSO(domain_id, ref lib_path) => Ok(Interrupt::LoadSO(domain_id, lib_path.clone())),
        Instruction::AddFFIFn(domain_id, function_id, ref function_name, ref arg_types, ref ret_type) => Ok(Interrupt::AddFFIFn(domain_id, function_id, function_name.clone(), arg_types.clone(), ret_type.clone())),
        Instruction::CallFFIFn(domain_id, function_id, arg_addr, n_arg_bytes, ret_addr) => Ok(Interrupt::CallFFIFn(domain_id, function_id, arg_addr, n_arg_bytes, ret_addr)),

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

    LoadSO(usize, String),
    AddFFIFn(usize, usize, String, Vec<Type>, Type),
    CallFFIFn(usize, usize, usize, usize, usize),

    Ok,
    EOF
}

fn extend_memory(memory: &mut Memory, n_bytes: usize) -> Result<Interrupt, String> {
    memory.extend_memory(n_bytes);
    return Ok(Interrupt::Ok)
}

fn extend_memory_to(memory: &mut Memory, n_bytes: usize) -> Result<Interrupt, String> {
    memory.extend_memory_to(n_bytes);
    return Ok(Interrupt::Ok)
}

fn delete_future(future_id: usize) -> Result<Interrupt, String> {
    return Ok(Interrupt::DeleteFuture(future_id));
}


fn write_to_symbol<T: ByteSerialisable>(memory: &mut Memory, symbol: usize, value: &T) -> Result<Interrupt, String> {
    memory.write(symbol, value);
    return Ok(Interrupt::Ok);
}

/// Copy the data in `source` to `dest`. Returns an error if `source` is undefined.
fn copy_symbol(memory: &mut Memory, source: usize, dest: usize, n: usize) -> Result<Interrupt, String> {
    memory.memcpy(source, dest, n)?;
    return Ok(Interrupt::Ok);
}

/// Local float trig trait so generic trig instruction helpers can call `.sin()` etc.
/// without relying on unstable/inapplicable bounds for arbitrary `T`.
trait FloatTrig: Copy {
    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn tan(self) -> Self;
    fn asin(self) -> Self;
    fn acos(self) -> Self;
    fn atan(self) -> Self;
}

impl FloatTrig for f32 {
    fn sin(self) -> Self { f32::sin(self) }
    fn cos(self) -> Self { f32::cos(self) }
    fn tan(self) -> Self { f32::tan(self) }
    fn asin(self) -> Self { f32::asin(self) }
    fn acos(self) -> Self { f32::acos(self) }
    fn atan(self) -> Self { f32::atan(self) }
}

impl FloatTrig for f64 {
    fn sin(self) -> Self { f64::sin(self) }
    fn cos(self) -> Self { f64::cos(self) }
    fn tan(self) -> Self { f64::tan(self) }
    fn asin(self) -> Self { f64::asin(self) }
    fn acos(self) -> Self { f64::acos(self) }
    fn atan(self) -> Self { f64::atan(self) }
}


/// Copy `n` bytes from actual memory address in `[ptr_index]` to dest
/// This is different from memcpy which uses offsets from the stack base pointer
fn ind(memory: &mut Memory, ptr_index: usize, dest: usize, n: usize) -> Result<Interrupt, String>{
    let source = memory.read_typed::<usize>(ptr_index);
    copy_symbol(memory, source, dest, n);
    return Ok(Interrupt::Ok);
}


/// Add the integers in `a` and `b` together, and put the result in `dest`.
/// Returns an error if either `a` or `b` is undefined, or does not contain an integer.
fn add_symbols<
    T: ByteParseable + ByteSerialisable + std::ops::Add<T, Output = T> + 'static,
>(
    memory: &mut Memory,
    a: usize,
    b: usize,
    dest: usize,
) -> Result<Interrupt, String> {
    let a_data = memory.read_typed::<T>(a);
    let b_data = memory.read_typed::<T>(b);
    let result = a_data + b_data;
    memory.write(dest, &result);
    Ok(Interrupt::Ok)
}

/// Subtract the integer in `b` from `a`, and put the result in `dest`.
fn subtract_symbols<
    T: ByteParseable + ByteSerialisable + std::ops::Sub<T, Output = T> + 'static,
>(
    memory: &mut Memory,
    a: usize,
    b: usize,
    dest: usize,
) -> Result<Interrupt, String> {
    let a_data = memory.read_typed::<T>(a);
    let b_data = memory.read_typed::<T>(b);
    let result = a_data - b_data;
    memory.write(dest, &result);
    Ok(Interrupt::Ok)
}

/// Multiply the integers in `a` and `b`, and put the result in `dest`.
fn multiply_symbols<
    T: ByteParseable + ByteSerialisable + std::ops::Mul<T, Output = T> + 'static,
>(
    memory: &mut Memory,
    a: usize,
    b: usize,
    dest: usize,
) -> Result<Interrupt, String> {
    let a_data = memory.read_typed::<T>(a);
    let b_data = memory.read_typed::<T>(b);
    let result = a_data * b_data;
    memory.write(dest, &result);
    Ok(Interrupt::Ok)
}

/// Divide the integer in `a` by `b`, and put the result in `dest`.
/// Returns an error if `b` is zero.
fn divide_symbols<
    T: ByteParseable
        + ByteSerialisable
        + std::ops::Div<T, Output = T>
        + PartialEq
        + Copy
        + 'static,
>(
    memory: &mut Memory,
    a: usize,
    b: usize,
    dest: usize,
) -> Result<Interrupt, String> {
    let a_data = memory.read_typed::<T>(a);
    let b_data = memory.read_typed::<T>(b);
    let result = a_data / b_data;
    memory.write(dest, &result);
    Ok(Interrupt::Ok)
}

/// Modulo the integer in `a` by `b`, and put the result in `dest`.
/// Returns an error if `b` is zero.
fn modulo_symbols<
    T: ByteParseable
        + ByteSerialisable
        + std::ops::Rem<T, Output = T>
        + PartialEq
        + Copy
        + 'static,
>(
    memory: &mut Memory,
    a: usize,
    b: usize,
    dest: usize,
) -> Result<Interrupt, String> {
    let a_data = memory.read_typed::<T>(a);
    let b_data = memory.read_typed::<T>(b);
   
    let result = a_data % b_data;   // Kinda hard to deal with div by zero so we can just throw a runtime error
    memory.write(dest, &result);
    Ok(Interrupt::Ok)
}

/// Minimum of `a` and `b`, put result in `dest`.
fn min_symbols<
    T: ByteParseable + ByteSerialisable + PartialOrd + 'static,
>(
    memory: &mut Memory,
    a: usize,
    b: usize,
    dest: usize,
) -> Result<Interrupt, String> {
    let a_data = memory.read_typed::<T>(a);
    let b_data = memory.read_typed::<T>(b);
    let result = if a_data <= b_data { a_data } else { b_data };
    memory.write(dest, &result);
    Ok(Interrupt::Ok)
}

/// Maximum of `a` and `b`, put result in `dest`.
fn max_symbols<
    T: ByteParseable + ByteSerialisable + PartialOrd + 'static,
>(
    memory: &mut Memory,
    a: usize,
    b: usize,
    dest: usize,
) -> Result<Interrupt, String> {
    let a_data = memory.read_typed::<T>(a);
    let b_data = memory.read_typed::<T>(b);
    let result = if a_data >= b_data { a_data } else { b_data };
    memory.write(dest, &result);
    Ok(Interrupt::Ok)
}

/// Check if the values in `a` and `b` are equal, and put the bool result in `dest`.
fn compare_equal<
    T: ByteParseable + ByteSerialisable + PartialEq + 'static,
>(
    memory: &mut Memory,
    a: usize,
    b: usize,
    dest: usize,
) -> Result<Interrupt, String> {
    let a_data = memory.read_typed::<T>(a);
    let b_data = memory.read_typed::<T>(b);
    let result = a_data == b_data;
    memory.write(dest, &result);
    Ok(Interrupt::Ok)
}

/// Check if `a` is greater than `b`, and put the bool result in `dest`.
fn compare_greater<
    T: ByteParseable + ByteSerialisable + PartialOrd + 'static,
>(
    memory: &mut Memory,
    a: usize,
    b: usize,
    dest: usize,
) -> Result<Interrupt, String> {
    let a_data = memory.read_typed::<T>(a);
    let b_data = memory.read_typed::<T>(b);
    let result = a_data > b_data;
    memory.write(dest, &result);
    Ok(Interrupt::Ok)
}

/// Check if `a` is less than `b`, and put the bool result in `dest`.
fn compare_lesser<
    T: ByteParseable + ByteSerialisable + PartialOrd + 'static,
>(
    memory: &mut Memory,
    a: usize,
    b: usize,
    dest: usize,
) -> Result<Interrupt, String> {
    let a_data = memory.read_typed::<T>(a);
    let b_data = memory.read_typed::<T>(b);
    let result = a_data < b_data;
    memory.write(dest, &result);
    Ok(Interrupt::Ok)
}

/// Perform fused multiply-add on the integers in `a`, `b`, and `c` (a * b + c), and put the result in `dest`.
/// Returns an error if any of `a`, `b`, or `c` is undefined, or does not contain an integer.
fn fma_symbols<T: ByteParseable + ByteSerialisable + std::ops::Add<T, Output = T> + std::ops::Mul<T, Output = T> + 'static>(
    memory: &mut Memory,
    a: usize,
    b: usize,
    c: usize,
    dest: usize,
) -> Result<Interrupt, String> {
    let a_data = memory.read_typed::<T>(a);
    let b_data = memory.read_typed::<T>(b);
    let c_data = memory.read_typed::<T>(c);
    let result = a_data * b_data + c_data;
    memory.write(dest, &result);
    return Ok(Interrupt::Ok);
}

/// Calculate the sine of the value in `a`, and put the result in `dest`.
fn sin_symbol<T: ByteParseable + ByteSerialisable + FloatTrig + 'static>(
    memory: &mut Memory,
    a: usize,
    dest: usize,
) -> Result<Interrupt, String> {
    let a_data = memory.read_typed::<T>(a);
    let result = a_data.sin();
    memory.write(dest, &result);
    Ok(Interrupt::Ok)
}

/// Calculate the cosine of the value in `a`, and put the result in `dest`.
fn cos_symbol<T: ByteParseable + ByteSerialisable + FloatTrig + 'static>(
    memory: &mut Memory,
    a: usize,
    dest: usize,
) -> Result<Interrupt, String> {
    let a_data = memory.read_typed::<T>(a);
    let result = a_data.cos();
    memory.write(dest, &result);
    Ok(Interrupt::Ok)
}

/// Calculate the tangent of the value in `a`, and put the result in `dest`.
fn tan_symbol<T: ByteParseable + ByteSerialisable + FloatTrig + 'static>(
    memory: &mut Memory,
    a: usize,
    dest: usize,
) -> Result<Interrupt, String> {
    let a_data = memory.read_typed::<T>(a);
    let result = a_data.tan();
    memory.write(dest, &result);
    Ok(Interrupt::Ok)
}

/// Calculate the arcsine of the value in `a`, and put the result in `dest`.
fn arcsin_symbol<T: ByteParseable + ByteSerialisable + FloatTrig + 'static>(
    memory: &mut Memory,
    a: usize,
    dest: usize,
) -> Result<Interrupt, String> {
    let a_data = memory.read_typed::<T>(a);
    let result = a_data.asin();
    memory.write(dest, &result);
    Ok(Interrupt::Ok)
}

/// Calculate the arccosine of the value in `a`, and put the result in `dest`.
fn arccos_symbol<T: ByteParseable + ByteSerialisable + FloatTrig + 'static>(
    memory: &mut Memory,
    a: usize,
    dest: usize,
) -> Result<Interrupt, String> {
    let a_data = memory.read_typed::<T>(a);
    let result = a_data.acos();
    memory.write(dest, &result);
    Ok(Interrupt::Ok)
}

/// Calculate the arctangent of the value in `a`, and put the result in `dest`.
fn arctan_symbol<T: ByteParseable + ByteSerialisable + FloatTrig + 'static>(
    memory: &mut Memory,
    a: usize,
    dest: usize,
) -> Result<Interrupt, String> {
    let a_data = memory.read_typed::<T>(a);
    let result = a_data.atan();
    memory.write(dest, &result);
    Ok(Interrupt::Ok)
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