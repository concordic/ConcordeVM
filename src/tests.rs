use std::fmt::Debug;
use std::rc::Rc;

use cloneable_any::CloneableAny;
use concordeisa::{instructions::Instruction};
use libffi::middle::Type;

use crate::memory::{ByteParseable, ByteSerialisable};

use crate::{CPU, Memory, Program, Scheduler};

fn execute(instructions: Vec<Instruction>) -> Result<Memory, String> {
    execute_entrypoint(instructions, 0)
}

fn execute_entrypoint(instructions: Vec<Instruction>, entrypoint: usize) -> Result<Memory, String> {
    let instructions: std::rc::Rc<Vec<Instruction>> = Rc::new(instructions);
    let program: Program = Program { instructions: instructions, pc: entrypoint };

    let mut scheduler = Scheduler::new();
    scheduler.run(program)?;

    return Ok(scheduler.get_coro(1).memory_dump());
}

fn check_symbol_eq<T: PartialEq + Debug + CloneableAny + ByteSerialisable + ByteParseable>(memory:Memory, symbol: usize, value: T) {
    let x =  memory.read_typed::<T>(symbol);
    assert_eq!(x, value)
}

// #[test]
// fn basic_io() {
//     let a = Symbol("a".to_string());
//     let b = Symbol("b".to_string());
//     let s = Symbol("s".to_string());
//     let n = Symbol("n".to_string());
//     let instructions = vec![
//         Instruction::WriteStringToSymbol(a.clone(), "stdio".to_string()),
//         Instruction::WriteBytesToSymbol(b.clone(), "test output\n".as_bytes().to_vec()),
//         Instruction::WriteIntToSymbol(n.clone(), 12),
//         Instruction::OpenStream(a, s.clone()),
//         Instruction::WriteStream(s.clone(), n, b),
//     ];

//     execute(instructions);
// }



/*
Foo:
    MemExtend 1000
    0 <- 1
    8 <- 2
    [16] = [0] + [8]
    ret 8 bytes at [16]

Main:
    MemExtend 1000
    
*/

#[test]
fn basic_arithmetic() -> Result<(), Box<dyn std::error::Error>> {
    let instructions = vec![
        Instruction::MemExtend(1000),
        Instruction::WriteIntToSymbol(0, 1i64),
        Instruction::WriteIntToSymbol(8, 2),
        Instruction::AddSymbols(0, 8, 16),
        Instruction::Return(16, 8)
    ];

    let memory = execute(instructions)?;
    print!("Done executing");
    check_symbol_eq(memory, 16, 3i64);
    Ok(())
}


#[test]
fn strings() -> Result<(), Box<dyn std::error::Error>> {
    let instructions = vec![
        Instruction::MemExtend(1000),
        Instruction::WriteStringToSymbol(0, String::from("Hello, world!")),
        Instruction::Return(0, 24)
    ];
    let memory = execute(instructions)?;
    print!("Done executing");
    check_symbol_eq(memory, 0, String::from("Hello, world!"));
    Ok(())
}


#[test]
fn function_calls() -> Result<(), Box<dyn std::error::Error>> {
    let instructions = vec![
        Instruction::MemExtend(100),
        Instruction::CreateCoroutine(9, 0, 0, 0),
        Instruction::Await(0, 0),
        Instruction::Return(0, 8),

        Instruction::MemExtend(1000),
        Instruction::WriteIntToSymbol(0, 1i64),
        Instruction::WriteIntToSymbol(8, 2i64),
        Instruction::AddSymbols(0, 8, 16),
        Instruction::Return(16, 8),

        Instruction::MemExtend(1000),
        Instruction::CreateCoroutine(4, 0, 0, 0),
        Instruction::Await(0, 0),
        Instruction::Return(0, 8)
    ];

    let memory = execute(instructions)?;
    print!("Done executing");
    check_symbol_eq(memory, 0, 3i64);
    Ok(())
}

#[test]
fn test_ffi() -> Result<(), Box<dyn std::error::Error>> {
    let instructions = vec![
        Instruction::MemExtend(100),    // 0    pseudomain
        Instruction::LoadSO(1, "./ffi.so".to_string()),
        Instruction::AddFFIFn(1, 1, "max".to_string(), vec![Type::u64(), Type::u64()], Type::u64()),
        Instruction::CreateCoroutine(12, 0, 0, 0),
        Instruction::Await(0, 0),
        Instruction::Return(0, 8),

        Instruction::MemExtend(1000),   // 6    add_ffi
        Instruction::WriteIntToSymbol(0, 10i64),
        Instruction::WriteIntToSymbol(8, 100i64),
        Instruction::CallFFIFn(1, 1, 0, 16, 16),
        Instruction::Await(16, 24),
        Instruction::Return(24, 8),

        Instruction::MemExtend(1000),   // 12 -- int main
        Instruction::CreateCoroutine(6, 0, 0, 0),
        Instruction::Await(0, 0),
        Instruction::Return(0, 8)
    ];

    let memory = execute(instructions)?;
    print!("Done executing");
    check_symbol_eq(memory, 0, 100i64);
    Ok(())
}