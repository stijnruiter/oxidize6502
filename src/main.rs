use oxidize6502::bus;
use oxidize6502::cpu::Cpu;

use std::time::Instant;


fn main() {
    println!("Execute 6502 functional tests..");
    let mut memory = [0; bus::MEMORY_SIZE];
    bus::load_binary(&mut memory, "tests\\6502_functional_test.bin", 0).unwrap();
    let mut cpu = Cpu::new();
    cpu.reset();
    cpu.program_counter = 0x0400;
    

    let mut previous_address: u16 = 0xFFFF; 
    let mut count = 0u8;
    let start = Instant::now();

    for _ in 0..100_000_000
    {
        cpu.run_step(&mut memory).unwrap();

        if cpu.program_counter == 0x3469 {
            println!("SUCCESS");
            break;
        }

        if cpu.program_counter == previous_address {
            count += 1;
            if count > 5 {
                println!("BREAK; infinite loop. PC=0x{:#04X}", cpu.program_counter);
                break;
            }
        } else {
            previous_address = cpu.program_counter;
            count = 0;
        }
    }
    let duration = start.elapsed();

    println!("Elapsed: {:?}", duration);
}


