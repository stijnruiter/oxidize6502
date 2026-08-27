mod bus;
mod cpu;

use bus::Bus;
use cpu::Cpu;

use crate::bus::Memory;

fn main() {
    let mut cycles = 0;

    let mut mem = Memory::new();
    mem.load([
        cpu::op_code::LDA_IMM, 0x55,
        cpu::op_code::LDA_IMM, 0x65,
        cpu::op_code::BREAK
    ]);
    let mut cpu = Cpu::new();
    cpu.reset();


    while cpu.is_running() {
        cycles += cpu.next_op(&mem);
        println!("Executing operation");
    }
    println!("Accumulator: 0x{:02X}", cpu.register_a);
    println!("Cylces: {}", cycles);
}
