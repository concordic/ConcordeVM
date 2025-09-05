#[macro_use]
mod errors;
mod cpu;
mod memory;
mod instructions;

fn main() {
    colog::init();

    let mut cpu = cpu::CPU::new();
    let inst = vec![
        instructions::Instruction::NoOp(),
        instructions::Instruction::WriteIntToSymbol(memory::Symbol("int_test_a".to_string()), 69),
        instructions::Instruction::WriteIntToSymbol(memory::Symbol("int_test_b".to_string()), 420),
        instructions::Instruction::WriteStringToSymbol(memory::Symbol("string_test_a".to_string()), "Hello World!".to_string()),
        instructions::Instruction::WriteStringToSymbol(memory::Symbol("string_test_b".to_string()), "Hello ConcordeVM!".to_string()),
        instructions::Instruction::CopySymbol(memory::Symbol("string_test_a".to_string()), memory::Symbol("string_test_c".to_string())),
        instructions::Instruction::AddSymbols(memory::Symbol("int_test_a".to_string()), memory::Symbol("int_test_b".to_string()), memory::Symbol("int_test_c".to_string())),
        instructions::Instruction::PrintSymbol(memory::Symbol("string_test_a".to_string())),
        instructions::Instruction::PrintSymbol(memory::Symbol("string_test_b".to_string())),
        instructions::Instruction::PrintSymbol(memory::Symbol("string_test_c".to_string())),
        instructions::Instruction::PrintSymbol(memory::Symbol("int_test_a".to_string())),
        instructions::Instruction::PrintSymbol(memory::Symbol("int_test_b".to_string())),
        instructions::Instruction::PrintSymbol(memory::Symbol("int_test_c".to_string())),
    ];
    cpu.load_instructions(&inst, &memory::Symbol("main".to_string()));

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
