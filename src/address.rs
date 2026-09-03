use std::fmt::Display;

use crate::{bus::Bus, cpu::Cpu};

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum AddressMode {
    Accumulator,

    Implied,
    Immediate,

    ZeroPage, ZeroPageX, ZeroPageY,
    Absolute, AbsoluteX, AbsoluteY,
    Indirect, IndirectX, IndirectY,

    Relative
}

impl Display for AddressMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct AddressResult
{
    pub address: u16,
    pub page_crossed: bool
}

impl From<u16> for AddressResult {
    fn from(value: u16) -> Self {
        Self {
            address: value,
            page_crossed: false
        }
    }
}

impl AddressMode {
    pub fn get_address(&self, cpu:&mut Cpu, bus: &impl Bus<u16>) -> AddressResult {
        match self {
            AddressMode::Accumulator => { 0u16.into() }
            AddressMode::Implied => { 0u16.into() },
            AddressMode::Immediate => {
                let address = cpu.program_counter;
                cpu.program_counter += 1;
                address.into()
            },
            AddressMode::ZeroPage => {
                let address = bus.read_byte(cpu.program_counter) as u16;
                cpu.program_counter += 1;
                address.into()
            },
            AddressMode::ZeroPageX => {
                let mut address = bus.read_byte(cpu.program_counter) as u16;
                cpu.program_counter += 1;
                address += cpu.register_x as u16;
                return (address & 0xFF).into(); // Masked; 0x0080 + 0x00FF = 0x007F (and not 0x017F)
            }, 
            AddressMode::ZeroPageY => {
                let mut address = bus.read_byte(cpu.program_counter) as u16;
                cpu.program_counter += 1;
                address += cpu.register_y as u16;
                (address & 0xFF).into() // Masked; 0x0080 + 0x00FF = 0x007F (and not 0x017F)
            },
            AddressMode::Absolute => { 
                let address = bus.read_word_little_endian(cpu.program_counter);
                cpu.program_counter += 2;
                address.into()
            },
            AddressMode::AbsoluteX => {
                let address = bus.read_word_little_endian(cpu.program_counter);
                cpu.program_counter += 2;

                let address_offset_x = address.wrapping_add(cpu.register_x as u16);
                AddressResult {
                    address: address_offset_x,
                    page_crossed: address & 0xFF00 != address_offset_x & 0xFF00
                }
            },
            AddressMode::AbsoluteY => {
                let address = bus.read_word_little_endian(cpu.program_counter);
                cpu.program_counter += 2;
                
                let address_offset_y = address.wrapping_add(cpu.register_y as u16);
                AddressResult {
                    address: address_offset_y,
                    page_crossed: address & 0xFF00 != address_offset_y & 0xFF00
                }
            },
            AddressMode::Indirect => {
                let pointer = bus.read_word_little_endian(cpu.program_counter);
                cpu.program_counter += 2;

                // emulate bug in original 6502, where lsb 0xXXFF causes the msb to wrap around page, reading 0xXX00 instead of next page
                let msb_address = if pointer & 0x00FF == 0x00FF { pointer & 0xFF00} else { pointer + 1};
                let low = bus.read_byte(pointer) as u16; 
                let high =  bus.read_byte(msb_address) as u16;
                let address = high << 8 | low;
                address.into()
            },
            AddressMode::IndirectX => {
                let mut indirect_address = bus.read_byte(cpu.program_counter) as u16;
                cpu.program_counter += 1;
                indirect_address += cpu.register_x as u16;
                indirect_address &= 0x00FF;
                
                let add_low = bus.read_byte(indirect_address) as u16;
                let add_high = bus.read_byte(indirect_address.wrapping_add(1) & 0xFF) as u16;
                (add_high << 8 | add_low).into()
            },
            AddressMode::IndirectY => {
                let zero_page_address = bus.read_byte(cpu.program_counter) as u16;
                cpu.program_counter += 1;

                let low = bus.read_byte(zero_page_address) as u16;
                let high = bus.read_byte(zero_page_address.wrapping_add(1) & 0x00FF) as u16;
                let indirect_address = high << 8 | low;

                let indirect_address_offset_y = indirect_address.wrapping_add(cpu.register_y as u16);
                AddressResult { 
                    address: indirect_address_offset_y, 
                    page_crossed: indirect_address & 0xFF00 != indirect_address_offset_y & 0xFF00
                }
            },
            AddressMode::Relative => {
                let relative_value = bus.read_byte(cpu.program_counter);
                cpu.program_counter += 1;

                let address_offset = relative_value.cast_signed() as i16;
                let address = cpu.program_counter.wrapping_add_signed(address_offset);
                AddressResult { 
                    address, 
                    page_crossed: address & 0xFF00 != cpu.program_counter & 0xFF00 
                }
            }
        }
    }
}


#[cfg(test)]
mod address_result_tests {
    use crate::address::AddressResult;
    
    #[test]
    fn into() {
        let address: u16 = 0x12;
        let result: AddressResult = address.into();
        assert_eq!(result.address, address);
        assert_eq!(result.page_crossed, false);
    }
}

#[cfg(test)]
mod address_modes_tests {
    use crate::cpu::Cpu;
    use crate::address::{AddressMode, AddressResult};
    use crate::instructions::op_codes as op;
    use test_case::test_case;

    #[test_case(0; "immediate_1")]
    #[test_case(1; "immediate_2")]
    fn test_modes(pc: u16) {
        let mem = [op::LDA_IMM, 0x05];
        let mut cpu = Cpu::new();
        cpu.program_counter = pc;
        assert_eq!(AddressMode::Immediate.get_address(&mut cpu, &mem), pc.into());
    }
    
    #[test_case(0 => AddressResult::from(op::LDA_IMM as u16); "zero page 1")]
    #[test_case(1 => AddressResult::from(0x05u16); "zero page 2")]
    fn test_zero_page(pc: u16) -> AddressResult {
        let mem = [op::LDA_IMM, 0x05];
        let mut cpu = Cpu::new();
        cpu.program_counter = pc;
        AddressMode::ZeroPage.get_address(&mut cpu, &mem)
    }


    #[test]
    fn test_zero_page_x() {
        let mem = [0x36, 0xF0];
        let mut cpu = Cpu::new();
        cpu.program_counter = 0;
        cpu.register_x = 0x15;
        cpu.register_y = 0x53;
        assert_eq!(AddressMode::ZeroPageX.get_address(&mut cpu, &mem), 0x004B.into(), "add by x");
        assert_eq!(AddressMode::ZeroPageX.get_address(&mut cpu, &mem), 0x0005.into(), "add by x overflow");
    }
    
    #[test]
    fn test_zero_page_y() {
        let mem = [0x36, 0xF0];
        let mut cpu = Cpu::new();
        cpu.program_counter = 0;
        cpu.register_x = 0x15;
        cpu.register_y = 0x53;
        assert_eq!(AddressMode::ZeroPageY.get_address(&mut cpu, &mem), 0x0089.into(), "add by y");
        assert_eq!(AddressMode::ZeroPageY.get_address(&mut cpu, &mem), 0x0043.into(), "add by y overflow");
    }

    
    #[test]
    fn test_absolute() {
        let mem = [0x36, 0xF0, 0xEF, 0xAB];
        let mut cpu = Cpu::new();
        cpu.program_counter = 0;
        cpu.register_x = 0x8E;
        cpu.register_y = 0x8F;
        assert_eq!(AddressMode::Absolute.get_address(&mut cpu, &mem), 0xF036.into(), "absolute little endian 1");
        assert_eq!(AddressMode::Absolute.get_address(&mut cpu, &mem), 0xABEF.into(), "absolute little endian 2");
    }

    
    #[test]
    fn test_absolute_x() {
        let mem = [0x36, 0xF0, 0xEF, 0xAB];
        let mut cpu = Cpu::new();
        cpu.program_counter = 0;
        cpu.register_x = 0x8E;
        cpu.register_y = 0x8F;
        assert_eq!(AddressMode::AbsoluteX.get_address(&mut cpu, &mem), AddressResult { address: 0xF0C4, page_crossed: false}, "absolute little endian 1");
        assert_eq!(cpu.program_counter, 2);
        assert_eq!(AddressMode::AbsoluteX.get_address(&mut cpu, &mem), AddressResult { address: 0xAC7D, page_crossed: true}, "absolute little endian 2");
        assert_eq!(cpu.program_counter, 4);
    }

    #[test]
    fn test_absolute_y() {
        let mem = [0x36, 0xF0, 0xEF, 0xAB];
        let mut cpu = Cpu::new();
        cpu.program_counter = 0;
        cpu.register_x = 0x8E;
        cpu.register_y = 0x8F;
        assert_eq!(AddressMode::AbsoluteY.get_address(&mut cpu, &mem), AddressResult { address: 0xF0C5, page_crossed: false}, "absolute little endian 1");
        assert_eq!(cpu.program_counter, 2);
        assert_eq!(AddressMode::AbsoluteY.get_address(&mut cpu, &mem), AddressResult { address: 0xAC7E, page_crossed: true}, "absolute little endian 2");
        assert_eq!(cpu.program_counter, 4);
    }

    #[test]
    fn test_indirect() {
        let mem = [0x08, 0x00, op::NOP, op::NOP, op::NOP, op::NOP, op::NOP, op::NOP, 0xAB, 0xCD];
        let mut cpu = Cpu::new();
        assert_eq!(AddressMode::Indirect.get_address(&mut cpu, &mem), AddressResult { address: 0xCDAB, page_crossed: false});
        assert_eq!(cpu.program_counter, 2);
    }

    
    #[test]
    fn test_indirect_page_wrap_around() {
        let mut mem = [0; 0x0200];
        mem[0x00FF] = 0x05;
        mem[0x0100] = 0x01;
        mem[0x0105] = 0xAA;
        mem[0x0106] = 0xBB;
        let mut cpu = Cpu::new();
        cpu.program_counter = 0x00FF;

        assert_eq!(AddressMode::Indirect.get_address(&mut cpu, &mem), 0xBBAA.into());
        assert_eq!(cpu.program_counter, 0x0101);
    }

    #[test_case(0x00, 0x02AB.into(); "zero x")]
    #[test_case(0x10, 0x05EF.into(); "non zero x")]
    #[test_case(0x7F, 0xCD80.into(); "boundary msb on wrap around")]
    #[test_case(0x80, 0x1FCD.into(); "full zero page wrap around")]
    fn indirect_x(reg_x: u8, address: AddressResult) {
        let mut memory = [0u8; 0x0300];
        memory[0x0080] = 0xAB;
        memory[0x0081] = 0x02;
        memory[0x0090] = 0xEF;
        memory[0x0091] = 0x05;
        memory[0x00FF] = 0x80;
        memory[0x0000] = 0xCD;
        memory[0x0001] = 0x1F;
        memory[0x0200] = 0x80;
        let mut cpu = Cpu::new();
        cpu.program_counter = 0x0200;
        cpu.register_x = reg_x;

        assert_eq!(AddressMode::IndirectX.get_address(&mut cpu, &memory), address);
        assert_eq!(cpu.program_counter, 0x0201);
        assert_eq!(cpu.register_x, reg_x);
    }

    #[test_case(0x00, 0x02AB.into(); "zero offset")]
    #[test_case(0x10, 0x02BB.into(); "within page offset")]
    #[test_case(0x54, 0x02FF.into(); "at the boundary, no page cross")]
    #[test_case(0x55, AddressResult { address: 0x0300, page_crossed: true }; "over the boundary, page crossed")]
    fn indirect_y(reg_y: u8, address: AddressResult) {
        let mut memory = [0u8; 0x0300];
        memory[0x0080] = 0xAB;
        memory[0x0081] = 0x02;
        memory[0x0200] = 0x80;
        let mut cpu = Cpu::new();
        cpu.program_counter = 0x0200;
        cpu.register_y = reg_y;

        assert_eq!(AddressMode::IndirectY.get_address(&mut cpu, &memory), address);
        assert_eq!(cpu.program_counter, 0x0201);
        assert_eq!(cpu.register_y, reg_y);

    }


}