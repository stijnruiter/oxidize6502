use crate::Bus;

#[repr(u8)]
enum StatusFlag {
    Negative =  0x80,
    Overflow =  0x40,
    Break =     0x10,
    Decimal =   0x08,
    Interrupt = 0x04,
    Zero =      0x02,
    Carry =     0x01,
}

pub mod op_code {
    pub const LDA_IMM: u8 = 0xA9;
    pub const LDA_ZER: u8 = 0xA5;

    pub const BREAK: u8 = 0x00;
}

pub struct Cpu {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,

    pub program_counter: u16,
    pub stack_pointer: u8,

    pub status: u8
}

impl Cpu {
    pub fn new() -> Self {
        Self { 
            register_a: 0, 
            register_x: 0, 
            register_y: 0, 
            
            program_counter: 0, 
            stack_pointer: 0, 
            
            status: 0
        }
    }

    pub fn reset(&mut self) {
        self.register_a =  0;
        self.register_x =  0; 
        self.register_y =  0; 
        
        self.program_counter = 0; 
        self.stack_pointer = 0; 
        self.status = 0;
    }

    pub fn next_op(&mut self, bus: &impl Bus<u16>) -> bool {
        let op = bus.read_byte(self.program_counter);
        self.program_counter += 1;

        match op {
            op_code::LDA_IMM => {
                let address = self.address_immediate();
                self.op_lda(address, bus);
            },
            op_code::LDA_ZER => {
                let address = self.address_zero_page(bus);
                self.op_lda(address, bus);
            },
            op_code::BREAK => { return false; }
            _ => todo!()
        }
        return true;
    }

    fn address_immediate(&mut self) -> u16 {
        let address = self.program_counter;
        self.program_counter += 1;
        return address;
    }

    fn address_zero_page(&mut self, bus: &impl Bus<u16>) -> u16 {
        let address = bus.read_byte(self.program_counter);
        self.program_counter += 1;
        return address as u16;
    }

    fn op_lda(&mut self, address: u16, bus: &impl Bus<u16>) {
        let value = bus.read_byte(address);
        self.set_status(StatusFlag::Negative, (value >> 7) == 1);
        self.set_status(StatusFlag::Zero, value == 0);
        self.register_a = value;
    }


    fn set_status(&mut self, key: StatusFlag, value: bool) {
        if value {
            self.status |= key as u8;
        } else {
            self.status &= !(key as u8)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{bus::Bus, cpu::{Cpu, StatusFlag}};

    #[test]
    fn lda_imm() {
        let mem: [u8; 3] = [0xA9, 0x12, 0x00];
        let mut cpu = Cpu::new();
        cpu.next_op(&mem);
        assert_eq!(cpu.register_a, 0x12);
        assert_eq!(cpu.program_counter, 2);
        assert_eq!(cpu.status, 0);
    }

    #[test]
    fn lda_imm_negative_status() {
        let mem: [u8; 3] = [0xA9, 0x85, 0x00];
        let mut cpu = Cpu::new();
        cpu.next_op(&mem);
        assert_eq!(cpu.register_a, 0x85);
        assert_eq!(cpu.program_counter, 2);
        assert_eq!(cpu.status, StatusFlag::Negative as u8);
    }

    #[test]
    fn lda_imm_zero_status() {
        let mem: [u8; 3] = [0xA9, 0x00, 0x00];
        let mut cpu = Cpu::new();
        cpu.next_op(&mem);
        assert_eq!(cpu.register_a, 0x00);
        assert_eq!(cpu.program_counter, 2);
        assert_eq!(cpu.status, StatusFlag::Zero as u8);
    }

    impl<const N: usize> Bus<u16> for [u8; N] {
        fn read_byte(&self, address: u16) -> u8 {
            self[address as usize]
        }
    
        fn write_byte(&mut self, address: u16, value: u8) {
            self[address as usize] = value;
        }
    }
}