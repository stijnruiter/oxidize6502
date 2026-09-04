// max size of 6502
pub const MEMORY_SIZE: usize = 0xFFFF + 1; 

#[allow(dead_code)]
pub trait Bus<T> {
    fn read_byte(&self, address: T) -> u8;
    fn read_word_little_endian(&self, address: T) -> u16;
    fn write_byte(&mut self, address: T, value: u8);
}

pub struct Memory {
    data: [u8; MEMORY_SIZE]
}

#[allow(dead_code)]
impl Memory {
    pub fn new() -> Self { 
        Self { 
            data: [0; MEMORY_SIZE]
        }
    }

    pub fn load<const N: usize>(&mut self, data: [u8; N]) {
        self.data[0..N].copy_from_slice(&data);
    }
}

impl Bus<u16> for Memory {
    fn read_byte(&self, address: u16) -> u8 {
        self.data[address as usize]
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        // TODO: range check
        self.data[address as usize] = value;
    }
    
    fn read_word_little_endian(&self, address: u16) -> u16 {
        read_word_little_endian(self, address)
    }
}

impl<const N: usize> Bus<u16> for [u8; N] {
    fn read_byte(&self, address: u16) -> u8 {
        self[address as usize]
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        self[address as usize] = value;
    }
    
    fn read_word_little_endian(&self, address: u16) -> u16 {
        read_word_little_endian(self, address)
    }
}

fn read_word_little_endian(bus: &impl Bus<u16>, address: u16) -> u16 {
    let low = bus.read_byte(address) as u16;
    let high = bus.read_byte(address.wrapping_add(1)) as u16;
    high << 8 | low
}

pub fn load_binary(mem: &mut [u8], path: &str, load_addr: u16) -> std::io::Result<()> {
    let data = std::fs::read(path)?;
    let start = load_addr as usize;
    let end = start + data.len();
    mem[start..end].copy_from_slice(&data);
    Ok(())
}