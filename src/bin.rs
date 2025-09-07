//! ConcordeVM's binary version.

use concordeisa::{instructions, memory};
use concordevm_lib::CPU;

fn main() {
    colog::init();

    let mut cpu = CPU::new();
    let main = vec![
        instructions::Instruction::WriteStringToSymbol(memory::Symbol("hello_world".to_string()), "Hello World! A > B!".to_string()),
        instructions::Instruction::WriteStringToSymbol(memory::Symbol("hello_concorde".to_string()), "Hello ConcordeVM! A < B!".to_string()),
        instructions::Instruction::WriteIntToSymbol(memory::Symbol("int_test_a".to_string()), 69),
        instructions::Instruction::WriteIntToSymbol(memory::Symbol("int_test_b".to_string()), 420),
        instructions::Instruction::Jump(memory::Symbol("branch_1".to_string())),
        instructions::Instruction::CompareGreater(memory::Symbol("int_test_a".to_string()), memory::Symbol("int_test_b".to_string()), memory::Symbol("result".to_string())),
        instructions::Instruction::JumpIfTrue(memory::Symbol("branch_a_g_b".to_string()), memory::Symbol("result".to_string())),
        instructions::Instruction::CompareLesser(memory::Symbol("int_test_a".to_string()), memory::Symbol("int_test_b".to_string()), memory::Symbol("result".to_string())),
        instructions::Instruction::JumpIfTrue(memory::Symbol("branch_a_l_b".to_string()), memory::Symbol("result".to_string())),
    ];
    let branch_1 = vec![
        instructions::Instruction::PrintSymbol(memory::Symbol("int_test_a".to_string())),
        instructions::Instruction::Return(),
        instructions::Instruction::PrintSymbol(memory::Symbol("int_test_b".to_string())),
    ];
    let branch_a_g_b = vec![
        instructions::Instruction::PrintSymbol(memory::Symbol("hello_world".to_string())),
    ];
    let branch_a_l_b = vec![
        instructions::Instruction::PrintSymbol(memory::Symbol("hello_concorde".to_string())),
    ];

    cpu.load_instructions(&main, &memory::Symbol("main".to_string()));
    cpu.load_instructions(&branch_1, &memory::Symbol("branch_1".to_string()));
    cpu.load_instructions(&branch_a_g_b, &memory::Symbol("branch_a_g_b".to_string()));
    cpu.load_instructions(&branch_a_l_b, &memory::Symbol("branch_a_l_b".to_string()));

    cpu.init_execution(&memory::Symbol("main".to_string()));
    
    let mut running = true;
    while running {
        let status = cpu.cycle();
        if let Err(e) = status {
            println!("CPU Crashed! Error: {}", e);
            return;
        }
        running = status.unwrap();
    }
}
