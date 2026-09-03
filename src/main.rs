mod bus;
mod cpu;

use crate::cpu::Cpu;


fn main() {
    let mut memory = [0; bus::MEMORY_SIZE];
    load_binary(&mut memory, "tests\\6502_functional_test.bin", 0).unwrap();
    let mut cpu = Cpu::new();
    cpu.reset();
    cpu.program_counter = 0x0400;
    

    let mut previous_address: u16 = 0xFFFF; 
    let mut count = 0u8;

    for i in 0..200_000
    {
        cpu.next_op(&mut memory).unwrap();

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

        if i % 1000 == 0 {
            println!("Instructions: {i} / 200_000");
            println!("MEM[0x0200] = {}", memory[0x0200]);
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