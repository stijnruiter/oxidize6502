mod bus;
mod cpu;

use crate::bus::Bus;
use crate::cpu::Cpu;
use crate::cpu::op_codes as op;

use crate::bus::Memory;


fn main() {
    let mut cycles = 0;

    let mut memory = Memory::new();
    memory.load([
        op::LDA_IMM, 0x55,
        op::LDA_IMM, 0x65
    ]);
    let mut cpu = Cpu::new();
    cpu.reset();


    for _ in 0..2 {
        cycles += cpu.next_op(&mut memory).unwrap();
        println!("Executing operation");
    }
    println!("Accumulator: 0x{:02X}", cpu.register_a);
    println!("Cycles: {}", cycles);
}
