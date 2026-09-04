use std::fmt::Display;

use crate::{address::AddressResult::Implied, bus::Bus, cpu::Cpu};

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
pub(crate) enum AddressResult
{
    Implied,
    Accumulator,
    Memory(u16),
    MemoryWithPageCross(u16)
}

impl AddressResult {
    pub fn address(&self) -> Option<u16> {
        match self {
            Implied => None,
            AddressResult::Accumulator => None,
            AddressResult::Memory(memory) => Some(*memory),
            AddressResult::MemoryWithPageCross(memory) => Some(*memory),
        }
    }

    pub fn has_crossed_page(&self) -> bool {
        matches!(self, AddressResult::MemoryWithPageCross(_))
    }
}

impl AddressMode {
    pub fn get_address(&self, cpu:&mut Cpu, bus: &impl Bus<u16>) -> AddressResult {
        match self {
            AddressMode::Accumulator => AddressResult::Accumulator,
            AddressMode::Implied => AddressResult::Implied,
            AddressMode::Immediate => {
                let address = AddressResult::Memory(cpu.program_counter);
                cpu.program_counter += 1;
                address
                
            },
            AddressMode::ZeroPage => {
                let address = bus.read_byte(cpu.program_counter) as u16;
                cpu.program_counter += 1;
                AddressResult::Memory(address)
            },
            AddressMode::ZeroPageX => {
                let mut address = bus.read_byte(cpu.program_counter) as u16;
                cpu.program_counter += 1;
                address += cpu.register_x as u16;
                AddressResult::Memory(address & 0xFF) // Masked; 0x0080 + 0x00FF = 0x007F (and not 0x017F)
            }, 
            AddressMode::ZeroPageY => {
                let mut address = bus.read_byte(cpu.program_counter) as u16;
                cpu.program_counter += 1;
                address += cpu.register_y as u16;
                AddressResult::Memory(address & 0xFF) // Masked; 0x0080 + 0x00FF = 0x007F (and not 0x017F)
            },
            AddressMode::Absolute => { 
                let address = bus.read_word_little_endian(cpu.program_counter);
                cpu.program_counter += 2;
                AddressResult::Memory(address)
            },
            AddressMode::AbsoluteX => {
                let address = bus.read_word_little_endian(cpu.program_counter);
                cpu.program_counter += 2;

                let address_offset_x = address.wrapping_add(cpu.register_x as u16);
                let page_crossed = address & 0xFF00 != address_offset_x & 0xFF00;
                if page_crossed {
                    AddressResult::MemoryWithPageCross(address_offset_x)
                } else {
                    AddressResult::Memory(address_offset_x)
                }
            },
            AddressMode::AbsoluteY => {
                let address = bus.read_word_little_endian(cpu.program_counter);
                cpu.program_counter += 2;
                
                let address_offset_y = address.wrapping_add(cpu.register_y as u16);
                let page_crossed = address & 0xFF00 != address_offset_y & 0xFF00;
                if page_crossed {
                    AddressResult::MemoryWithPageCross(address_offset_y)
                } else {
                    AddressResult::Memory(address_offset_y)
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
                AddressResult::Memory(address)
            },
            AddressMode::IndirectX => {
                let mut indirect_address = bus.read_byte(cpu.program_counter) as u16;
                cpu.program_counter += 1;
                indirect_address += cpu.register_x as u16;
                indirect_address &= 0x00FF;
                
                let add_low = bus.read_byte(indirect_address) as u16;
                let add_high = bus.read_byte(indirect_address.wrapping_add(1) & 0xFF) as u16;
                AddressResult::Memory(add_high << 8 | add_low)
            },
            AddressMode::IndirectY => {
                let zero_page_address = bus.read_byte(cpu.program_counter) as u16;
                cpu.program_counter += 1;

                let low = bus.read_byte(zero_page_address) as u16;
                let high = bus.read_byte(zero_page_address.wrapping_add(1) & 0x00FF) as u16;
                let indirect_address = high << 8 | low;

                let indirect_address_offset_y = indirect_address.wrapping_add(cpu.register_y as u16);
                let page_crossed =  indirect_address & 0xFF00 != indirect_address_offset_y & 0xFF00;
                if page_crossed {
                    AddressResult::MemoryWithPageCross(indirect_address_offset_y)
                } else {
                    AddressResult::Memory(indirect_address_offset_y)
                }
            },
            AddressMode::Relative => {
                let relative_value = bus.read_byte(cpu.program_counter);
                cpu.program_counter += 1;

                let address_offset = relative_value.cast_signed() as i16;
                let address = cpu.program_counter.wrapping_add_signed(address_offset);
                let page_crossed = address & 0xFF00 != cpu.program_counter & 0xFF00;
                if page_crossed {
                    AddressResult::MemoryWithPageCross(address)
                } else {
                    AddressResult::Memory(address)
                }
            }
        }
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
        assert_eq!(AddressMode::Immediate.get_address(&mut cpu, &mem), AddressResult::Memory(pc));
    }
    
    #[test_case(0 => AddressResult::Memory(op::LDA_IMM as u16); "zero page 1")]
    #[test_case(1 => AddressResult::Memory(0x05); "zero page 2")]
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
        assert_eq!(AddressMode::ZeroPageX.get_address(&mut cpu, &mem), AddressResult::Memory(0x004B), "add by x");
        assert_eq!(AddressMode::ZeroPageX.get_address(&mut cpu, &mem), AddressResult::Memory(0x0005), "add by x overflow");
    }
    
    #[test]
    fn test_zero_page_y() {
        let mem = [0x36, 0xF0];
        let mut cpu = Cpu::new();
        cpu.program_counter = 0;
        cpu.register_x = 0x15;
        cpu.register_y = 0x53;
        assert_eq!(AddressMode::ZeroPageY.get_address(&mut cpu, &mem), AddressResult::Memory(0x0089), "add by y");
        assert_eq!(AddressMode::ZeroPageY.get_address(&mut cpu, &mem), AddressResult::Memory(0x0043), "add by y overflow");
    }

    
    #[test]
    fn test_absolute() {
        let mem = [0x36, 0xF0, 0xEF, 0xAB];
        let mut cpu = Cpu::new();
        cpu.program_counter = 0;
        cpu.register_x = 0x8E;
        cpu.register_y = 0x8F;
        assert_eq!(AddressMode::Absolute.get_address(&mut cpu, &mem), AddressResult::Memory(0xF036), "absolute little endian 1");
        assert_eq!(AddressMode::Absolute.get_address(&mut cpu, &mem), AddressResult::Memory(0xABEF), "absolute little endian 2");
    }

    
    #[test]
    fn test_absolute_x() {
        let mem = [0x36, 0xF0, 0xEF, 0xAB];
        let mut cpu = Cpu::new();
        cpu.program_counter = 0;
        cpu.register_x = 0x8E;
        cpu.register_y = 0x8F;
        assert_eq!(AddressMode::AbsoluteX.get_address(&mut cpu, &mem), AddressResult::Memory(0xF0C4), "absolute little endian 1");
        assert_eq!(cpu.program_counter, 2);
        assert_eq!(AddressMode::AbsoluteX.get_address(&mut cpu, &mem), AddressResult::MemoryWithPageCross(0xAC7D), "absolute little endian 2");
        assert_eq!(cpu.program_counter, 4);
    }

    #[test]
    fn test_absolute_y() {
        let mem = [0x36, 0xF0, 0xEF, 0xAB];
        let mut cpu = Cpu::new();
        cpu.program_counter = 0;
        cpu.register_x = 0x8E;
        cpu.register_y = 0x8F;
        assert_eq!(AddressMode::AbsoluteY.get_address(&mut cpu, &mem), AddressResult::Memory(0xF0C5), "absolute little endian 1");
        assert_eq!(cpu.program_counter, 2);
        assert_eq!(AddressMode::AbsoluteY.get_address(&mut cpu, &mem), AddressResult::MemoryWithPageCross(0xAC7E), "absolute little endian 2");
        assert_eq!(cpu.program_counter, 4);
    }

    #[test]
    fn test_indirect() {
        let mem = [0x08, 0x00, op::NOP, op::NOP, op::NOP, op::NOP, op::NOP, op::NOP, 0xAB, 0xCD];
        let mut cpu = Cpu::new();
        assert_eq!(AddressMode::Indirect.get_address(&mut cpu, &mem), AddressResult::Memory(0xCDAB));
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

        assert_eq!(AddressMode::Indirect.get_address(&mut cpu, &mem), AddressResult::Memory(0xBBAA));
        assert_eq!(cpu.program_counter, 0x0101);
    }

    #[test_case(0x00, AddressResult::Memory(0x02AB); "zero x")]
    #[test_case(0x10, AddressResult::Memory(0x05EF); "non zero x")]
    #[test_case(0x7F, AddressResult::Memory(0xCD80); "boundary msb on wrap around")]
    #[test_case(0x80, AddressResult::Memory(0x1FCD); "full zero page wrap around")]
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

    #[test_case(0x00, AddressResult::Memory(0x02AB); "zero offset")]
    #[test_case(0x10, AddressResult::Memory(0x02BB); "within page offset")]
    #[test_case(0x54, AddressResult::Memory(0x02FF); "at the boundary, no page cross")]
    #[test_case(0x55, AddressResult::MemoryWithPageCross(0x0300); "over the boundary, page crossed")]
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