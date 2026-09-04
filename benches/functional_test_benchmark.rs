use std::hint::black_box;

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use oxidize6502::{bus, cpu::Cpu};

fn run_test(memory: &mut [u8; bus::MEMORY_SIZE]) {
    let mut cpu = Cpu::new();
    cpu.reset();
    cpu.program_counter = 0x0400;
    for _ in 0..100_000_000
    {
        let _ = cpu.run_step(memory);
        if cpu.program_counter == 0x3469 {
            return;
        }
    }
    panic!()
}

fn functional_test_benchmark(c: &mut Criterion) {
    let mut memory = [0; bus::MEMORY_SIZE];
    bus::load_binary(&mut memory, "tests\\6502_functional_test.bin", 0).unwrap();
    c.bench_function("Klaus2m5 Functional Tests", |b| {
        b.iter_batched(
            || memory.clone(),           // setup: runs before each iteration, NOT timed
            move |mut cloned_data| run_test(black_box(&mut cloned_data)), // this IS timed
            BatchSize::LargeInput,
        )
    });
}
criterion_group!(benches, functional_test_benchmark);
criterion_main!(benches);