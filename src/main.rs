mod bus;
mod cpu;

use bus::Bus;
use cpu::Cpu;

use crate::bus::Memory;

fn main() {
    let mut mem = Memory::new();
    mem.load([
        cpu::op_code::LDA_IMM, 0x55,
        cpu::op_code::LDA_IMM, 0x55,
        cpu::op_code::BREAK
    ]);
    let mut cpu = Cpu::new();
    cpu.reset();

    while cpu.next_op(&mem) {
        println!("Executing operation");
    }
    let accum = cpu.register_a;
    println!("Accumulator: 0x{:02X}", accum);

}
