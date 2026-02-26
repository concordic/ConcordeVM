use std::fmt::Debug;
use std::rc::Rc;

use cloneable_any::CloneableAny;
use concordeisa::{instructions::Instruction};

use crate::memory::{ByteParseable, ByteSerialisable};

use crate::{CPU, Program, Scheduler};

fn execute(instructions: Vec<Instruction>) -> CPU {
    execute_entrypoint(instructions, 0)
}

fn execute_entrypoint(instructions: Vec<Instruction>, entrypoint: usize) -> CPU {
    let instructions: std::rc::Rc<Vec<Instruction>> = Rc::new(instructions);
    let program: Program = Program { instructions: instructions, pc: entrypoint };

    let scheduler = Scheduler::new();
    scheduler.run(program);

    return scheduler;
}

fn check_symbol_eq<T: PartialEq + Debug + CloneableAny + ByteSerialisable + ByteParseable>(cpu: CPU, symbol: usize, value: T) {
    let x =  cpu.get_memory().read_typed::<T>(symbol);
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
fn basic_arithmetic() {
    let instructions = vec![
        Instruction::WriteIntToSymbol(0, 1i64),
        Instruction::WriteIntToSymbol(8, 2),
        Instruction::AddSymbols(0, 8, 16),
        Instruction::Return(16, 8)
    ];

    let cpu = execute(instructions);
    print!("Done executing");
    check_symbol_eq(cpu, 16, 3i64);
}


#[test]
fn strings() {
    let instructions = vec![
        Instruction::WriteStringToSymbol(0, String::from("Hello, world!")),
        Instruction::Return(0, 24)
    ];
    let cpu = execute(instructions);
    print!("Done executing");
    check_symbol_eq(cpu, 0, String::from("Hello, world!"));
}