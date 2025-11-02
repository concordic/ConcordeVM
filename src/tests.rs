use std::fmt::Debug;

use cloneable_any::CloneableAny;
use concordeisa::{instructions::Instruction, memory::Symbol};

use crate::{CPU};

fn execute(instructions: Vec<Instruction>) -> CPU {
    execute_entrypoint(instructions, &Symbol("main".to_string()))
}

fn execute_entrypoint(instructions: Vec<Instruction>, entrypoint: &Symbol) -> CPU {
    let mut cpu = CPU::new();
    cpu.load_instructions(&instructions, entrypoint);
    cpu.init_execution(entrypoint);
    let mut running = true;
    while running {
        match cpu.cycle() {
            Ok(b) => running = b,
            Err(e) => panic!("Test failed during execution! {}", e),
        }
    }
    cpu
}

fn check_symbol_eq<T: PartialEq + Debug + CloneableAny>(cpu: CPU, symbol: &Symbol, value: T) {
    match cpu.get_memory().read_typed::<T>(symbol) {
        Ok(a) => assert_eq!(*a, value),
        Err(e) => panic!("Test failed during evaluation! {}", e),
    }
}

#[test]
fn basic_io() {
    let a = Symbol("a".to_string());
    let b = Symbol("b".to_string());
    let s = Symbol("s".to_string());
    let n = Symbol("n".to_string());
    let instructions = vec![
        Instruction::WriteStringToSymbol(a.clone(), "stdio".to_string()),
        Instruction::WriteBytesToSymbol(b.clone(), "test output".as_bytes().to_vec()),
        Instruction::WriteIntToSymbol(n.clone(), 11),
        Instruction::OpenStream(a, s.clone()),
        Instruction::WriteStream(s.clone(), n, b),
    ];

    execute(instructions);
}

#[test]
fn basic_arithmetic() {
    let a = Symbol("a".to_string());
    let b = Symbol("b".to_string());
    let c = Symbol("c".to_string());
    let instructions = vec![
        Instruction::WriteIntToSymbol(a.clone(), 1),
        Instruction::WriteIntToSymbol(b.clone(), 2),
        Instruction::AddSymbols(a.clone(), b.clone(), c.clone()),
    ];

    let cpu = execute(instructions);
    check_symbol_eq(cpu, &c, 3i64);
}
