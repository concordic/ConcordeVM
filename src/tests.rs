use std::fmt::Debug;
use std::rc::Rc;

use cloneable_any::CloneableAny;
use concordeisa::{instructions::Instruction};

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
