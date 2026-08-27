// max size of 6502
const MOMORY_SIZE: usize = 0xFFFF; 

#[allow(dead_code)]
pub trait Bus<T> {
    fn read_byte(&self, address: T) -> u8;
    fn write_byte(&mut self, address: T, value: u8);
}

pub struct Memory {
    data: [u8; MOMORY_SIZE]
}

impl Memory {
    pub fn new() -> Self { 
        Self { 
            data: [0; MOMORY_SIZE]
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
}
