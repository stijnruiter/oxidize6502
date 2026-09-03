use oxidize6502::bus;
use oxidize6502::cpu::Cpu;


fn main() {
    println!("Execute 6502 functional tests..");
    let mut memory = [0; bus::MEMORY_SIZE];
    load_binary(&mut memory, "tests\\6502_functional_test.bin", 0).unwrap();
    let mut cpu = Cpu::new();
    cpu.reset();
    cpu.program_counter = 0x0400;
    

    let mut previous_address: u16 = 0xFFFF; 
    let mut count = 0u8;

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
}


fn load_binary(mem: &mut [u8], path: &str, load_addr: u16) -> std::io::Result<()> {
    let data = std::fs::read(path)?;
    let start = load_addr as usize;
    let end = start + data.len();
    mem[start..end].copy_from_slice(&data);
    Ok(())
}