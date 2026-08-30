use crate::{Bus};
/**
 * https://6502.org/users/obelisk/6502/reference.html
 */

#[repr(u8)]
#[allow(dead_code)]
enum StatusFlag {
    Negative =  0x80,
    Overflow =  0x40,
    Break =     0x10,
    Decimal =   0x08,
    Interrupt = 0x04,
    Zero =      0x02,
    Carry =     0x01,
}

#[derive(Clone, Copy, PartialEq)]
enum AddressMode {
    Accumulator,

    Implied,
    Immediate,

    ZeroPage, ZeroPageX, ZeroPageY,
    Absolute, AbsoluteX, AbsoluteY,
    Indirect, IndirectX, IndirectY
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
enum Mnemonic {
    ADC, AND, ASL, BCC, BCS, BEQ, BIT, BMI, BNE, BPL, BRK, BVC, BVS, CLC,
    CLD, CLI, CLV, CMP, CPX, CPY, DEC, DEX, DEY, EOR, INC, INX, INY, JMP,
    JSR, LDA, LDX, LDY, LSR, NOP, ORA, PHA, PHP, PLA, PLP, ROL, ROR, RTI,
    RTS, SBC, SEC, SED, SEI, STA, STX, STY, TAX, TAY, TSX, TXA, TXS, TYA
}

#[derive(Clone, Copy)]
struct Instruction {
    address_mode: AddressMode,
    mnemonic: Mnemonic,
    cycles: u8,
    can_cross_page: bool
}

#[derive(Debug, PartialEq)]
struct AddressResult
{
    address: u16,
    page_crossed: bool
}

impl From<u16> for AddressResult {
    fn from(value: u16) -> Self {
        Self {
            address: value,
            page_crossed: false
        }
    }
}

macro_rules! instructions {
    ($( $name:ident : $code:expr => ($mnemenic:ident, $addr:ident, $cycles:expr, $cross:expr) ),* $(,)?) => {
        pub mod op_codes {
            $(
                pub const $name: u8 = $code;
            )*
        }

        static INSTRUCTION_SET: [Option<Instruction>; 0x100] = {
            let mut table: [Option<Instruction>; 0x100] = [None; 0x100];
            $(
                table[op_codes::$name as usize] =
                    Some(Instruction {
                        address_mode: AddressMode::$addr,
                        mnemonic: Mnemonic::$mnemenic,
                        cycles: $cycles,
                        can_cross_page: $cross
                    });
            )*
            table
        };
    };
}

instructions!{
    BRK:     0x00 => (BRK, Implied,     7, false),
    NOP:     0xEA => (NOP, Implied,     2, false),
    CLC:     0x18 => (CLC, Implied,     2, false),
    CLD:     0xD8 => (CLD, Implied,     2, false),
    CLI:     0x58 => (CLI, Implied,     2, false),
    CLV:     0xB8 => (CLV, Implied,     2, false),

    INC_ZER: 0xE6 => (INC, ZeroPage,    5, false),
    INC_ZEX: 0xF6 => (INC, ZeroPageX,   6, false),
    INC_ABS: 0xEE => (INC, Absolute,    6, false),
    INC_ABX: 0xFE => (INC, AbsoluteY,   7, false),

    INX:     0xE8 => (INX, Implied,     2, false),
    INY:     0xC8 => (INY, Implied,     2, false),

    LDX_IMM: 0xA2 => (LDX, Immediate,   2, false),
    LDX_ZER: 0xA6 => (LDX, ZeroPage,    3, false),
    LDX_ZEY: 0xB6 => (LDX, ZeroPageY,   4, false),
    LDX_ABS: 0xAE => (LDX, Absolute,    4, false),
    LDX_ABY: 0xBE => (LDX, AbsoluteY,   4, true),
    
    LDY_IMM: 0xA0 => (LDY, Immediate,   2, false),
    LDY_ZER: 0xA4 => (LDY, ZeroPage,    3, false),
    LDY_ZEX: 0xB4 => (LDY, ZeroPageX,   4, false),
    LDY_ABS: 0xAC => (LDY, Absolute,    4, false),
    LDY_ABX: 0xBC => (LDY, AbsoluteX,   4, true),
    
    LDA_IMM: 0xA9 => (LDA, Immediate,   2, false),
    LDA_ZER: 0xA5 => (LDA, ZeroPage,    3, false),
    LDA_ZEX: 0xB5 => (LDA, ZeroPageX,   4, false),
    LDA_ABS: 0xAD => (LDA, Absolute,    4, false),
    LDA_ABX: 0xBD => (LDA, AbsoluteX,   4, true),
    LDA_ABY: 0xB9 => (LDA, AbsoluteY,   4, true),
    LDA_INX: 0xA1 => (LDA, IndirectX,   6, false),
    LDA_INY: 0xB1 => (LDA, IndirectY,   5, true),
    
    AND_IMM: 0x29 => (AND, Immediate,   2, false),
    AND_ZER: 0x25 => (AND, ZeroPage,    3, false),
    AND_ZEX: 0x35 => (AND, ZeroPageX,   4, false),
    AND_ABS: 0x2D => (AND, Absolute,    4, false),
    AND_ABX: 0x3D => (AND, AbsoluteX,   4, true),
    AND_ABY: 0x39 => (AND, AbsoluteY,   4, true),
    AND_INX: 0x21 => (AND, IndirectX,   6, false),
    AND_INY: 0x31 => (AND, IndirectY,   5, true),
    
    ASL_ACC: 0x0A => (ASL, Accumulator, 2, false),
    ASL_ZER: 0x06 => (ASL, ZeroPage,    5, false),
    ASL_ZEX: 0x16 => (ASL, ZeroPageX,   6, false),
    ASL_ABS: 0x0E => (ASL, Absolute,    6, false),
    ASL_ABX: 0x1E => (ASL, AbsoluteX,   7, false),
    
    CMP_IMM: 0xC9 => (CMP, Immediate,   2, false),
    CMP_ZER: 0xC5 => (CMP, ZeroPage,    3, false),
    CMP_ZEX: 0xD5 => (CMP, ZeroPageX,   4, false),
    CMP_ABS: 0xCD => (CMP, Absolute,    4, false),
    CMP_ABX: 0xDD => (CMP, AbsoluteX,   4, true),
    CMP_ABY: 0xD9 => (CMP, AbsoluteY,   4, true),
    CMP_INX: 0xC1 => (CMP, IndirectX,   6, false),
    CMP_INY: 0xD1 => (CMP, IndirectY,   5, true),
    
    CPX_IMM: 0xE0 => (CPX, Immediate,   2, false),
    CPX_ZER: 0xE4 => (CPX, ZeroPage,    3, false),
    CPX_ABS: 0xEC => (CPX, Absolute,    4, false),
    
    CPY_IMM: 0xC0 => (CPY, Immediate,   2, false),
    CPY_ZER: 0xC4 => (CPY, ZeroPage,    3, false),
    CPY_ABS: 0xCC => (CPY, Absolute,    4, false),
    
    DEC_ZER: 0xC6 => (DEC, ZeroPage,    5, false),
    DEC_ZEX: 0xD6 => (DEC, ZeroPageX,   6, false),
    DEC_ABS: 0xCE => (DEC, Absolute,    6, false),
    DEC_ABX: 0xDE => (DEC, AbsoluteX,   7, false),
    
    DEX:     0xCA => (DEX, Implied,     2, false),  
    DEY:     0x88 => (DEY, Implied,     2, false), 
    
    EOR_IMM: 0x49 => (EOR, Immediate,   2, false),
    EOR_ZER: 0x45 => (EOR, ZeroPage,    3, false),
    EOR_ZEX: 0x55 => (EOR, ZeroPageX,   4, false),
    EOR_ABS: 0x4D => (EOR, Absolute,    4, false),
    EOR_ABX: 0x5D => (EOR, AbsoluteX,   4, true),
    EOR_ABY: 0x59 => (EOR, AbsoluteY,   4, true),
    EOR_INX: 0x41 => (EOR, IndirectX,   6, false),
    EOR_INY: 0x51 => (EOR, IndirectY,   5, true),
    
    JMP_ABS: 0x4C => (JMP, Absolute,    3, false),
    JMP_IND: 0x6C => (JMP, Indirect,    5, false),

    LSR_ACC: 0x4A => (LSR, Accumulator, 2, false), 
    LSR_ZER: 0x46 => (LSR, ZeroPage,    5, false), 
    LSR_ZEX: 0x56 => (LSR, ZeroPageX,   6, false), 
    LSR_ABS: 0x4E => (LSR, Absolute,    6, false), 
    LSR_ABX: 0x5E => (LSR, AbsoluteX,   7, false),

    ORA_IMM: 0x09 => (ORA, Immediate,   2, false),
    ORA_ZER: 0x05 => (ORA, ZeroPage,    3, false),
    ORA_ZEX: 0x15 => (ORA, ZeroPageX,   4, false),
    ORA_ABS: 0x0D => (ORA, Absolute,    4, false),
    ORA_ABX: 0x1D => (ORA, AbsoluteX,   4, true), 
    ORA_ABY: 0x19 => (ORA, AbsoluteY,   4, true), 
    ORA_INX: 0x01 => (ORA, IndirectX,   6, false),
    ORA_INY: 0x11 => (ORA, IndirectY,   5, true), 

    ROL_ACC: 0x2A => (ROL, Accumulator, 2, false), 
    ROL_ZER: 0x26 => (ROL, ZeroPage,    5, false), 
    ROL_ZEX: 0x36 => (ROL, ZeroPageX,   6, false), 
    ROL_ABS: 0x2E => (ROL, Absolute,    6, false), 
    ROL_ABX: 0x3E => (ROL, AbsoluteX,   7, false),

    ROR_ACC: 0x6A => (ROR, Accumulator, 2, false), 
    ROR_ZER: 0x66 => (ROR, ZeroPage,    5, false), 
    ROR_ZEX: 0x76 => (ROR, ZeroPageX,   6, false), 
    ROR_ABS: 0x6E => (ROR, Absolute,    6, false), 
    ROR_ABX: 0x7E => (ROR, AbsoluteX,   7, false),

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

    pub fn has_breaked(&self) -> bool {
        self.status & (StatusFlag::Break as u8) == (StatusFlag::Break as u8)
    }

    pub fn next_op(&mut self, bus: &mut impl Bus<u16>) -> Result<u8, String> {
        let next_instruction = bus.read_byte(self.program_counter);
        self.program_counter += 1;

        match &INSTRUCTION_SET[next_instruction as usize] {
            Some(instruction) => {
                return Ok(self.execute_op(instruction, bus))
            },
            None => {
                return Err(format!("Operation {:02X} not supported", next_instruction))
            }
        }
    }

    fn execute_op(&mut self, instruction: &Instruction, bus: &mut impl Bus<u16>) -> u8 {
        use Mnemonic::*;
        
        let address_result = self.get_address(instruction.address_mode, bus);
        match instruction.mnemonic {
            ADC => { todo!(); }, 
            AND => { 
                let value = bus.read_byte(address_result.address);
                self.register_a &= value;
                self.set_status(StatusFlag::Negative, self.register_a >> 7 == 1);
                self.set_status(StatusFlag::Zero, self.register_a == 0);
            }, 
            ASL => {
                if instruction.address_mode == AddressMode::Accumulator {
                    self.register_a = self.asl_value(self.register_a);
                } 
                else {
                    let mut value = bus.read_byte(address_result.address);
                    value = self.asl_value(value);
                    bus.write_byte(address_result.address, value);
                }
            }, 
            BCC => { todo!(); }, 
            BCS => { todo!(); }, 
            BEQ => { todo!(); }, 
            BIT => { todo!(); }, 
            BMI => { todo!(); }, 
            BNE => { todo!(); }, 
            BPL => { todo!(); }, 
            BRK => { self.set_status(StatusFlag::Break, true); },
            BVC => { todo!(); }, 
            BVS => { todo!(); }, 
            CLC => { self.set_status(StatusFlag::Carry, false); },
            CLD => { self.set_status(StatusFlag::Decimal, false); },
            CLI => { self.set_status(StatusFlag::Interrupt, false); },
            CLV => { self.set_status(StatusFlag::Overflow, false); },
            CMP => { self.compare(self.register_a, bus.read_byte(address_result.address)) }, 
            CPX => { self.compare(self.register_x, bus.read_byte(address_result.address)) }, 
            CPY => { self.compare(self.register_y, bus.read_byte(address_result.address)) }, 
            DEC => { 
                let value = bus.read_byte(address_result.address);
                bus.write_byte(address_result.address, self.decrement_value(value)); 
            }, 
            DEX => { self.register_x = self.decrement_value(self.register_x); }, 
            DEY => { self.register_y = self.decrement_value(self.register_y); }, 
            EOR => { 
                let value = bus.read_byte(address_result.address);
                let result = self.register_a ^ value;
                self.set_status(StatusFlag::Negative, result >> 7 == 1);
                self.set_status(StatusFlag::Zero, result == 0);
                self.register_a = result;
            }, 
            INC => { 
                let value = bus.read_byte(address_result.address);
                bus.write_byte(address_result.address, self.increment_value(value)); 
            }, 
            INX => { self.register_x = self.increment_value(self.register_x); }, 
            INY => { self.register_y = self.increment_value(self.register_y); }, 
            JMP => { self.program_counter = address_result.address },
            JSR => { todo!(); }, 
            LDA => { self.register_a = self.load_value(address_result.address, bus); },
            LDX => { self.register_x = self.load_value(address_result.address, bus); },
            LDY => { self.register_y = self.load_value(address_result.address, bus); },
            LSR => { 
                if instruction.address_mode == AddressMode::Accumulator {
                    self.register_a = self.lsr_value(self.register_a);
                } 
                else {
                    let mut value = bus.read_byte(address_result.address);
                    value = self.lsr_value(value);
                    bus.write_byte(address_result.address, value);
                } 
            }, 
            NOP => { /* do nothing */ }
            ORA => { 
                let value = bus.read_byte(address_result.address);
                self.register_a |= value;
                self.set_status(StatusFlag::Zero, self.register_a == 0);
                self.set_status(StatusFlag::Negative, self.register_a >> 7 == 1);
             }, 
            PHA => { todo!(); }, 
            PHP => { todo!(); }, 
            PLA => { todo!(); }, 
            PLP => { todo!(); }, 
            ROL => {
                if instruction.address_mode == AddressMode::Accumulator {
                    self.register_a = self.rol_value(self.register_a);
                } 
                else {
                    let mut value = bus.read_byte(address_result.address);
                    value = self.rol_value(value);
                    bus.write_byte(address_result.address, value);
                } 
            }, 
            ROR => { 
                if instruction.address_mode == AddressMode::Accumulator {
                    self.register_a = self.ror_value(self.register_a);
                } 
                else {
                    let mut value = bus.read_byte(address_result.address);
                    value = self.ror_value(value);
                    bus.write_byte(address_result.address, value);
                }  }, 
            RTI => { todo!(); },
            RTS => { todo!(); }, 
            SBC => { todo!(); }, 
            SEC => { todo!(); }, 
            SED => { todo!(); }, 
            SEI => { todo!(); }, 
            STA => { todo!(); }, 
            STX => { todo!(); }, 
            STY => { todo!(); }, 
            TAX => { todo!(); }, 
            TAY => { todo!(); }, 
            TSX => { todo!(); }, 
            TXA => { todo!(); }, 
            TXS => { todo!(); }, 
            TYA => { todo!(); }
        }

        if instruction.can_cross_page && address_result.page_crossed {
            instruction.cycles + 1
        } else {
            instruction.cycles
        }
    }

    fn load_value(&mut self, address: u16, bus: &impl Bus<u16>) -> u8 {
        let value = bus.read_byte(address);
        self.set_status(StatusFlag::Negative, (value >> 7) == 1);
        self.set_status(StatusFlag::Zero, value == 0);
        return value;
    }
    
    fn asl_value(&mut self, value: u8) -> u8 {
        let new_value = value << 1;
        self.set_status(StatusFlag::Carry, value >> 7 == 1);
        self.set_status(StatusFlag::Negative, new_value >> 7 == 1);
        self.set_status(StatusFlag::Zero, new_value == 0);
        return new_value;
    }
    
    fn lsr_value(&mut self, value: u8) -> u8 {
        let new_value = value >> 1;
        self.set_status(StatusFlag::Carry, value & 1 == 1);
        self.set_status(StatusFlag::Negative, false);
        self.set_status(StatusFlag::Zero, new_value == 0);
        return new_value;
    }

    fn rol_value(&mut self, value: u8) -> u8 {
        let mut new_value = value << 1;
        if self.is_set(StatusFlag::Carry) {
            new_value |= 1;
        }

        self.set_status(StatusFlag::Carry, value >> 7 == 1);
        self.set_status(StatusFlag::Negative, new_value >> 7 == 1);
        self.set_status(StatusFlag::Zero, new_value == 0);
        new_value
    }

    fn ror_value(&mut self, value: u8) -> u8 {
        let mut new_value = value >> 1;
        if self.is_set(StatusFlag::Carry) {
            new_value |= 1 << 7;
        }

        self.set_status(StatusFlag::Carry, value & 1 == 1);
        self.set_status(StatusFlag::Negative, new_value >> 7 == 1);
        self.set_status(StatusFlag::Zero, new_value == 0);
        new_value
    }

    fn get_address(&mut self, mode: AddressMode, bus: &impl Bus<u16>) -> AddressResult {
        match mode {
            AddressMode::Accumulator => { 0u16.into() }
            AddressMode::Implied => { 0u16.into() },
            AddressMode::Immediate => {
                let address = self.program_counter;
                self.program_counter += 1;
                address.into()
            },
            AddressMode::ZeroPage => {
                let address = bus.read_byte(self.program_counter) as u16;
                self.program_counter += 1;
                address.into()
            },
            AddressMode::ZeroPageX => {
                let mut address = bus.read_byte(self.program_counter) as u16;
                self.program_counter += 1;
                address += self.register_x as u16;
                return (address & 0xFF).into(); // Masked; 0x0080 + 0x00FF = 0x007F (and not 0x017F)
            }, 
            AddressMode::ZeroPageY => {
                let mut address = bus.read_byte(self.program_counter) as u16;
                self.program_counter += 1;
                address += self.register_y as u16;
                (address & 0xFF).into() // Masked; 0x0080 + 0x00FF = 0x007F (and not 0x017F)
            },
            AddressMode::Absolute => { self.fetch_word_at_pc(bus).into() },
            AddressMode::AbsoluteX => {
                let address = self.fetch_word_at_pc(bus);
                let address_offset_x = address.wrapping_add(self.register_x as u16);
                AddressResult {
                    address: address_offset_x,
                    page_crossed: address & 0xFF00 != address_offset_x & 0xFF00
                }
            },
            AddressMode::AbsoluteY => {
                let address = self.fetch_word_at_pc(bus);
                let address_offset_y = address.wrapping_add(self.register_y as u16);
                AddressResult {
                    address: address_offset_y,
                    page_crossed: address & 0xFF00 != address_offset_y & 0xFF00
                }
            },
            AddressMode::Indirect => {
                let low = bus.read_byte(self.program_counter) as u16;
                // emulate bug in original 6502, where lsb 0xXXFF causes the msb to wrap around page, reading 0xXX00 instead of next page
                let msb_address = if self.program_counter & 0x00FF == 0x00FF { self.program_counter & 0xFF00} else { self.program_counter + 1};
                let high =  bus.read_byte(msb_address) as u16;
                let address = high << 8 | low;

                self.program_counter += 2;

                crate::Cpu::read_word_little_endian(bus, address).into()
            }
            _ => { todo!(); }
        }
    }

    fn fetch_word_at_pc(&mut self, bus: &impl Bus<u16>) -> u16 {
        let address = crate::Cpu::read_word_little_endian(bus, self.program_counter);
        self.program_counter += 2;
        address
    }

    fn read_word_little_endian(bus: &impl Bus<u16>, address: u16) -> u16 {
        let low = bus.read_byte(address) as u16;
        let high = bus.read_byte(address + 1) as u16;
        high << 8 | low
    }

    fn set_status(&mut self, key: StatusFlag, value: bool) {
        if value {
            self.status |= key as u8;
        } else {
            self.status &= !(key as u8)
        }
    }

    fn is_set(&self, key: StatusFlag) -> bool {
        let status_bit = key as u8;
        self.status & status_bit == status_bit
    }
    
    fn increment_value(&mut self, value: u8) -> u8 {
        let increment_value = value.wrapping_add(1);
        self.set_status(StatusFlag::Negative, increment_value >> 7 == 1);
        self.set_status(StatusFlag::Zero, increment_value == 0);
        return increment_value;
    }

    fn decrement_value(&mut self, value: u8) -> u8 {
        let decrement_value = value.wrapping_sub(1);
        self.set_status(StatusFlag::Negative, decrement_value >> 7 == 1);
        self.set_status(StatusFlag::Zero, decrement_value == 0);
        return decrement_value;
    }
    
    fn compare(&mut self, register_value: u8, value: u8) {
        self.set_status(StatusFlag::Carry, register_value >= value);
        self.set_status(StatusFlag::Zero, register_value == value);
        self.set_status(StatusFlag::Negative, register_value.wrapping_sub(value) >> 7 == 1);
    }
}

#[cfg(test)]
mod load_register_tests {
    use crate::cpu::Cpu;
    use crate::cpu::op_codes::*;

    macro_rules! test_load_immediate {
        ($op:ident, $register:ident) => {
            mod $register {
                use crate::cpu::{Cpu, StatusFlag};
                use crate::cpu::op_codes::*;

                #[test]
                fn immediate() {
                    let mut memory = [$op, 0x12];
                    let mut cpu = Cpu::new();
                    assert_eq!(cpu.next_op(&mut memory).unwrap(), 2);
                    assert_eq!(cpu.$register, 0x12);
                    assert_eq!(cpu.program_counter, 2);
                    assert_eq!(cpu.status, 0);
                }

                #[test]
                fn immediate_negative_status() {
                    let mut memory = [$op, 0x85];
                    let mut cpu = Cpu::new();
                    assert_eq!(cpu.next_op(&mut memory).unwrap(), 2);
                    assert_eq!(cpu.$register, 0x85);
                    assert_eq!(cpu.program_counter, 2);
                    assert_eq!(cpu.status, StatusFlag::Negative as u8);
                }

                #[test]
                fn immediate_zero_status() {
                    let mut memory = [$op, 0x00];
                    let mut cpu = Cpu::new();
                    assert_eq!(cpu.next_op(&mut memory).unwrap(), 2);
                    assert_eq!(cpu.$register, 0x00);
                    assert_eq!(cpu.program_counter, 2);
                    assert_eq!(cpu.status, StatusFlag::Zero as u8);
                }
            }
        };
    }
    test_load_immediate!(LDA_IMM, register_a);
    test_load_immediate!(LDX_IMM, register_x);
    test_load_immediate!(LDY_IMM, register_y);

    #[test]
    fn lda_zero_page() {
        let mut mem = [LDA_ZER, 0x05, NOP, NOP, BRK, 0x33];
        let mut cpu = Cpu::new();
        assert_eq!(cpu.next_op(&mut mem).unwrap(), 3);
        assert_eq!(cpu.register_a, 0x33);
        assert_eq!(cpu.program_counter, 2);
        assert_eq!(cpu.status, 0);
    }
}

#[cfg(test)]
mod increment_instruction_tests {
    use crate::cpu::{Cpu, StatusFlag, op_codes as op};

    #[test]
    fn increment_register_x() {
        
        let mut memory = [op::INX, op::INX, op::INX];
        let mut cpu = Cpu::new();
        cpu.reset();
        (cpu.register_a, cpu.register_x, cpu.register_y) = (0xEE, 0xFE, 0xDF);

        assert_eq!(cpu.next_op(&mut memory).unwrap(), 2);        
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y), (0xEE, 0xFF, 0xDF));
        assert_eq!(cpu.status, StatusFlag::Negative as u8);

        assert_eq!(cpu.next_op(&mut memory).unwrap(), 2);
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y), (0xEE, 0x00, 0xDF));
        assert_eq!(cpu.status, StatusFlag::Zero as u8);

        assert_eq!(cpu.next_op(&mut memory).unwrap(), 2);
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y), (0xEE, 0x01, 0xDF));
        assert_eq!(cpu.status, 0);
    }

    
    #[test]
    fn decrement_register_x() {
        
        let mut memory = [op::DEX, op::DEX, op::DEX];
        let mut cpu = Cpu::new();
        cpu.reset();
        (cpu.register_a, cpu.register_x, cpu.register_y) = (0xEE, 0x01, 0xDF);

        assert_eq!(cpu.next_op(&mut memory).unwrap(), 2);        
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y), (0xEE, 0x00, 0xDF));
        assert_eq!(cpu.status, StatusFlag::Zero as u8);

        assert_eq!(cpu.next_op(&mut memory).unwrap(), 2);
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y), (0xEE, 0xFF, 0xDF));
        assert_eq!(cpu.status, StatusFlag::Negative as u8);
    }

    
    #[test]
    fn increment_register_y() {
        
        let mut memory = [op::INY, op::INY, op::INY];
        let mut cpu = Cpu::new();
        cpu.reset();
        (cpu.register_a, cpu.register_x, cpu.register_y) = (0xEE, 0xAB, 0xFE);

        assert_eq!(cpu.next_op(&mut memory).unwrap(), 2);        
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y), (0xEE, 0xAB, 0xFF));
        assert_eq!(cpu.status, StatusFlag::Negative as u8);

        assert_eq!(cpu.next_op(&mut memory).unwrap(), 2);
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y), (0xEE, 0xAB, 0x00));
        assert_eq!(cpu.status, StatusFlag::Zero as u8);

        assert_eq!(cpu.next_op(&mut memory).unwrap(), 2);
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y), (0xEE, 0xAB, 0x01));
        assert_eq!(cpu.status, 0);
    }
    
    #[test]
    fn decrement_register_y() {
        
        let mut memory = [op::DEY, op::DEY, op::DEY];
        let mut cpu = Cpu::new();
        cpu.reset();
        (cpu.register_a, cpu.register_x, cpu.register_y) = (0xEE, 0xDF, 0x01);

        assert_eq!(cpu.next_op(&mut memory).unwrap(), 2);        
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y), (0xEE, 0xDF, 0x00));
        assert_eq!(cpu.status, StatusFlag::Zero as u8);

        assert_eq!(cpu.next_op(&mut memory).unwrap(), 2);
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y), (0xEE, 0xDF, 0xFF));
        assert_eq!(cpu.status, StatusFlag::Negative as u8);
    }

    #[test]
    fn increment_memory_zero_page() {
        let mut memory = [op::INC_ZER, 0x02, 0xFE];
        let mut cpu = Cpu::new();
        cpu.reset();

        assert_eq!(cpu.next_op(&mut memory).unwrap(), 5);
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y), (0, 0, 0));
        assert_eq!(memory, [op::INC_ZER, 0x02, 0xFF]);
        assert_eq!(cpu.status, StatusFlag::Negative as u8);

        cpu.reset();
        assert_eq!(cpu.next_op(&mut memory).unwrap(), 5);
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y), (0, 0, 0));
        assert_eq!(memory, [op::INC_ZER, 0x02, 0x00]);
        assert_eq!(cpu.status, StatusFlag::Zero as u8);
        
        cpu.reset();
        assert_eq!(cpu.next_op(&mut memory).unwrap(), 5);
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y), (0, 0, 0));
        assert_eq!(memory, [op::INC_ZER, 0x02, 0x01]);
        assert_eq!(cpu.status, 0);
    }

    #[test]
    fn increment_memory_zero_page_x() {
        let mut memory = [op::INC_ZEX, 0x02, op::NOP, op::NOP, 0xFE, op::NOP];
        let mut cpu = Cpu::new();
        cpu.reset();
        cpu.register_x = 0x02;

        assert_eq!(cpu.next_op(&mut memory).unwrap(), 6);
        assert_eq!(memory, [op::INC_ZEX, 0x02, op::NOP, op::NOP, 0xFF, op::NOP]);
        assert_eq!(cpu.status, StatusFlag::Negative as u8);
    }

    #[test]
    fn decrement_memory_zero_page() {
        let mut memory = [op::DEC_ZER, 0x02, 0x01];
        let mut cpu = Cpu::new();
        cpu.reset();

        assert_eq!(cpu.next_op(&mut memory).unwrap(), 5);
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y), (0, 0, 0));
        assert_eq!(memory, [op::DEC_ZER, 0x02, 0x00]);
        assert_eq!(cpu.status, StatusFlag::Zero as u8);

        cpu.reset();
        assert_eq!(cpu.next_op(&mut memory).unwrap(), 5);
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y), (0, 0, 0));
        assert_eq!(memory, [op::DEC_ZER, 0x02, 0xFF]);
        assert_eq!(cpu.status, StatusFlag::Negative as u8);
    }
    
    #[test]
    fn decrement_memory_zero_page_x() {
        let mut memory = [op::DEC_ZEX, 0x02, op::NOP, op::NOP, 0xFE, op::NOP];
        let mut cpu = Cpu::new();
        cpu.reset();
        cpu.register_x = 0x02;

        assert_eq!(cpu.next_op(&mut memory).unwrap(), 6);
        assert_eq!(memory, [op::DEC_ZEX, 0x02, op::NOP, op::NOP, 0xFD, op::NOP]);
        assert_eq!(cpu.status, StatusFlag::Negative as u8);
    }
}

#[cfg(test)]
mod direct_instruction_tests {
    use crate::cpu::{Cpu, StatusFlag, op_codes::{self as op}};
    use test_case::test_case;

    #[test]
    fn break_set_status() {
        let mut mem = [op::LDA_IMM, 0x00, op::BRK];
        let mut cpu = Cpu::new();

        cpu.next_op(&mut mem).unwrap();
        assert_eq!(cpu.has_breaked(), false);
        
        cpu.next_op(&mut mem).unwrap();
        assert_eq!(cpu.has_breaked(), true);
    }

    #[test_case(op::CLC, StatusFlag::Carry; "clear carry bit")] 
    #[test_case(op::CLD, StatusFlag::Decimal; "clear decimal bit")] 
    #[test_case(op::CLI, StatusFlag::Interrupt; "clear interrupt bit")] 
    #[test_case(op::CLV, StatusFlag::Overflow; "clear overflow bit")] 
    fn clear_codes(op_code: u8, status_bit: StatusFlag) {
        let mut memory = [op_code];
        let mut cpu = Cpu::new();
        let status_flag = status_bit as u8;

        cpu.reset();
        assert_eq!(cpu.next_op(&mut memory).unwrap(), 2);
        assert_eq!(cpu.status, 0);

        cpu.reset();
        cpu.status = status_flag;
        assert_eq!(cpu.next_op(&mut memory).unwrap(), 2);
        assert_eq!(cpu.status, 0);

        cpu.reset();
        cpu.status = 0xFF;
        assert_eq!(cpu.next_op(&mut memory).unwrap(), 2);
        assert_eq!(cpu.status, !status_flag);
    }

    #[test]
    fn nop_code() {
        const N: usize = 6;
        let mut mem: [u8; N] = [op::NOP, op::NOP, op::NOP, op::NOP, op::NOP, op::NOP ];
        let mut cpu = Cpu::new();
        let mut cycles = 0u8;
        cpu.reset();
        
        for _ in 0..N {
            cycles += cpu.next_op(&mut mem).unwrap();
        }
 
        assert_eq!(cycles, 2 * N as u8, "cycles executed");
        assert_eq!(cpu.program_counter, N as u16, "current program counter");
        assert_eq!(cpu.register_a, 0, "current a register");
        assert_eq!(cpu.register_x, 0, "current x register");
        assert_eq!(cpu.register_y, 0, "current y register");
        assert_eq!(cpu.stack_pointer, 0, "current stack pointer");
        assert_eq!(cpu.status, 0, "current status");
    }
}

#[cfg(test)]
mod operation_tests {
    use crate::cpu::{Cpu, StatusFlag, op_codes::{self as op}};
    use test_case::test_case;

    #[test_case(0b1100_1000, 0b1001_0000, StatusFlag::Carry as u8 | StatusFlag::Negative as u8; "asl zero page carry negative")]
    #[test_case(0b1001_0000, 0b0010_0000, StatusFlag::Carry as u8; "asl zero page carry only")]
    #[test_case(0b0010_0000, 0b0100_0000, 0; "asl zer page regular")]
    #[test_case(0b0100_0000, 0b1000_0000, StatusFlag::Negative as u8; "asl zero page negative only")]
    #[test_case(0b1000_0000, 0b0000_0000, StatusFlag::Carry as u8 | StatusFlag::Zero as u8; "asl zero page carry zero")]
    #[test_case(0b0000_0000, 0b0000_0000, StatusFlag::Zero as u8; "asl zero page zero only")]
    fn asl_accumulator(accumulator_before: u8, expected_after: u8, expected_status: u8) {
        let mut memory = [op::ASL_ACC];
        let mut cpu = Cpu::new();
        cpu.reset();
        cpu.register_a = accumulator_before;
        
        assert_eq!(cpu.next_op(&mut memory).unwrap(), 2, "Operation cycles");
        assert_eq!(cpu.register_a, expected_after, "Accumulator");
        assert_eq!(cpu.program_counter, 1, "Program counter");
        assert_eq!(cpu.status, expected_status, "CPU status");
    }
    
    #[test_case(0b1100_1000, 0b1001_0000, StatusFlag::Carry as u8 | StatusFlag::Negative as u8; "asl zero page carry negative")]
    #[test_case(0b1001_0000, 0b0010_0000, StatusFlag::Carry as u8; "asl zero page carry only")]
    #[test_case(0b0010_0000, 0b0100_0000, 0; "asl zer page regular")]
    #[test_case(0b0100_0000, 0b1000_0000, StatusFlag::Negative as u8; "asl zero page negative only")]
    #[test_case(0b1000_0000, 0b0000_0000, StatusFlag::Carry as u8 | StatusFlag::Zero as u8; "asl zero page carry zero")]
    #[test_case(0b0000_0000, 0b0000_0000, StatusFlag::Zero as u8; "asl zero page zero only")]
    fn asl_zero_page(before: u8, expected_after: u8, expected_status: u8) {
        let mut memory = [op::ASL_ZER, 0x02, before];
        let mut cpu = Cpu::new();
        cpu.reset();
        
        assert_eq!(cpu.next_op(&mut memory).unwrap(), 5, "Operation cycles");
        assert_eq!(memory[0x02], expected_after, "Memory value");
        assert_eq!(cpu.program_counter, 2, "Program counter");
        assert_eq!(cpu.status, expected_status, "CPU status");
        assert_eq!(cpu.register_a, 0, "Accumulator");
    }
    
    #[test_case(0b1100_1000, 0b0110_0100, 0u8; "Regular bit shift")]
    #[test_case(0b1100_1001, 0b0110_0100, StatusFlag::Carry as u8; "Carry bit shift")]
    #[test_case(0b0000_0001, 0b0000_0000, StatusFlag::Carry as u8 | StatusFlag::Zero as u8; "Carry and zero")]
    #[test_case(0b0000_0000, 0b0000_0000, StatusFlag::Zero as u8; "Zero")]
    fn lsr_accumulator(before: u8, expected_after: u8, expected_status: u8) {
        let mut memory = [op::LSR_ACC];
        let mut cpu = Cpu::new();
        cpu.reset();
        cpu.set_status(StatusFlag::Negative, true);
        cpu.register_a = before;
        
        assert_eq!(cpu.next_op(&mut memory).unwrap(), 2, "Operation cycles");
        assert_eq!(cpu.register_a, expected_after, "Accumulator");
        assert_eq!(cpu.program_counter, 1, "Program counter");
        assert_eq!(cpu.status, expected_status, "CPU status flags");
    }

    #[test_case(0b1100_1000, 0b0110_0100, 0u8; "Regular bit shift")]
    #[test_case(0b1100_1001, 0b0110_0100, StatusFlag::Carry as u8; "Carry bit shift")]
    #[test_case(0b0000_0001, 0b0000_0000, StatusFlag::Carry as u8 | StatusFlag::Zero as u8; "Carry and zero")]
    #[test_case(0b0000_0000, 0b0000_0000,StatusFlag::Zero as u8; "Zero")]
    fn lsr_zero_page(before: u8, expected_after: u8, expected_status: u8) {
        let mut memory = [op::LSR_ZER, 0x02, before];
        let mut cpu = Cpu::new();
        cpu.reset();
        cpu.set_status(StatusFlag::Negative, true);
        
        assert_eq!(cpu.next_op(&mut memory).unwrap(), 5, "Operation cycles");
        assert_eq!(memory[0x02], expected_after, "Memory value");
        assert_eq!(cpu.program_counter, 2, "Program counter");
        assert_eq!(cpu.status, expected_status, "CPU status");
        assert_eq!(cpu.register_a, 0, "");
    }

    // ORA, ROL, ROR
    #[test_case(0b1001_0110, 0b0000_0000, 0b1001_0110, StatusFlag::Negative as u8; "unchanged with zero")]
    #[test_case(0b0000_0000, 0b1001_0110, 0b1001_0110, StatusFlag::Negative as u8; "zero accumulator")]
    #[test_case(0b0110_1001, 0b0001_0110, 0b0111_1111, 0; "regular without status flags")]
    #[test_case(0, 0, 0, StatusFlag::Zero as u8; "zero")]
    fn ora(accumulator_before: u8, value_before: u8, expected_result: u8, expected_status: u8) {
        let mut memory = [op::ORA_IMM, value_before];
        let mut cpu = Cpu::new();
        cpu.reset();
        cpu.set_status(StatusFlag::Negative, true);
        cpu.register_a = accumulator_before;

        assert_eq!(cpu.next_op(&mut memory).unwrap(), 2, "Operation cycles");
        assert_eq!(cpu.register_a, expected_result, "Accumulator");
        assert_eq!(cpu.status, expected_status, "CPU status flags");
    }
    
    #[test_case(0b0000_0000, false, 0b0000_0000, StatusFlag::Zero as u8; "unchanged")]
    #[test_case(0b0000_0000, true, 0b0000_0001, 0; "carry transferred to value")]
    #[test_case(0b1010_1010, true, 0b0101_0101, StatusFlag::Carry as u8; "carry correctly set")]
    #[test_case(0b0000_1010, false, 0b0001_0100, 0; "flags cleared")]
    #[test_case(0b0100_1010, false, 0b1001_0100, StatusFlag::Negative as u8; "negative is set")]
    fn rol_accumulator(accumulator_before: u8, carry_bit: bool, expected_value: u8, expected_status: u8) {
        let mut memory = [op::ROL_ACC];
        let mut cpu = Cpu::new();
        cpu.reset();
        cpu.set_status(StatusFlag::Negative, true);
        cpu.set_status(StatusFlag::Carry, carry_bit);
        cpu.register_a = accumulator_before;

        assert_eq!(cpu.next_op(&mut memory).unwrap(), 2, "Operation cycles");
        assert_eq!(cpu.register_a, expected_value, "Accumulator");
        assert_eq!(cpu.status, expected_status, "CPU status flags");
    }
    
    #[test_case(0b0000_0000, false, 0b0000_0000, StatusFlag::Zero as u8; "unchanged")]
    #[test_case(0b0000_0000, true, 0b0000_0001, 0; "carry transferred to value")]
    #[test_case(0b1010_1010, true, 0b0101_0101, StatusFlag::Carry as u8; "carry correctly set")]
    #[test_case(0b0000_1010, false, 0b0001_0100, 0; "flags cleared")]
    #[test_case(0b0100_1010, false, 0b1001_0100, StatusFlag::Negative as u8; "negative is set")]
    fn rol_memory(value_before: u8, carry_bit: bool, expected_value: u8, expected_status: u8) {
        let mut memory = [op::ROL_ZER, 0x02, value_before];
        let mut cpu = Cpu::new();
        cpu.reset();
        cpu.set_status(StatusFlag::Negative, true);
        cpu.set_status(StatusFlag::Carry, carry_bit);
        cpu.register_a = 0xAB;

        assert_eq!(cpu.next_op(&mut memory).unwrap(), 5, "Operation cycles");
        assert_eq!(memory[2], expected_value, "Memory value");
        assert_eq!(cpu.status, expected_status, "CPU status flags");
        assert_eq!(cpu.register_a, 0xAB);
    }

    #[test_case(0b0000_0000, false, 0b0000_0000, StatusFlag::Zero as u8; "unchanged")]
    #[test_case(0b0000_0000, true, 0b1000_0000, StatusFlag::Negative as u8; "carry transferred to value")]
    #[test_case(0b1010_1010, true, 0b1101_0101, StatusFlag::Negative as u8; "negative correctly set, carry applied")]
    #[test_case(0b0000_1010, false, 0b0000_0101, 0; "flags cleared")]
    #[test_case(0b0100_1011, true, 0b1010_0101, StatusFlag::Carry as u8 | StatusFlag::Negative as u8; "carry and negative are set")]
    #[test_case(0b0100_1011, false, 0b0010_0101, StatusFlag::Carry as u8; "carry is set")]
    fn ror_accumulator(accumulator_before: u8, carry_bit: bool, expected_value: u8, expected_status: u8) {
        let mut memory = [op::ROR_ACC];
        let mut cpu = Cpu::new();
        cpu.reset();
        cpu.set_status(StatusFlag::Negative, true);
        cpu.set_status(StatusFlag::Carry, carry_bit);
        cpu.register_a = accumulator_before;

        assert_eq!(cpu.next_op(&mut memory).unwrap(), 2, "Operation cycles");
        assert_eq!(cpu.register_a, expected_value, "Accumulator");
        assert_eq!(cpu.status, expected_status, "CPU status flags");
    }

    #[test_case(0b0000_0000, false, 0b0000_0000, StatusFlag::Zero as u8; "unchanged")]
    #[test_case(0b0000_0000, true, 0b1000_0000, StatusFlag::Negative as u8; "carry transferred to value")]
    #[test_case(0b1010_1010, true, 0b1101_0101, StatusFlag::Negative as u8; "negative correctly set, carry applied")]
    #[test_case(0b0000_1010, false, 0b0000_0101, 0; "flags cleared")]
    #[test_case(0b0100_1011, true, 0b1010_0101, StatusFlag::Carry as u8 | StatusFlag::Negative as u8; "carry and negative are set")]
    #[test_case(0b0100_1011, false, 0b0010_0101, StatusFlag::Carry as u8; "carry is set")]
    fn ror_memory(memory_before: u8, carry_bit: bool, expected_value: u8, expected_status: u8) {
        let mut memory = [op::ROR_ZER, 0x02, memory_before];
        let mut cpu = Cpu::new();
        cpu.reset();
        cpu.set_status(StatusFlag::Negative, true);
        cpu.set_status(StatusFlag::Carry, carry_bit);
        cpu.register_a = 0xAB;

        assert_eq!(cpu.next_op(&mut memory).unwrap(), 5, "Operation cycles");
        assert_eq!(memory[2], expected_value, "Memory value");
        assert_eq!(cpu.status, expected_status, "CPU status flags");
        assert_eq!(cpu.register_a, 0xAB, "Accumulator");
    }

    #[test]
    fn logical_and() {
        let mut memory = [
            op::AND_IMM, 0b_1010_1010, op::AND_ZER, 0x0B, op::AND_ABS, 0x0C, 0x00, 
            op::AND_IMM, 0b_1111_0000, op::NOP, op::NOP, 
            0b_1001_0110, // 0x0B
            0b_0000_1111]; // 0x0C
        let mut cpu = Cpu::new();
        cpu.reset();
        cpu.register_a = 0xFF;

        assert_eq!(cpu.next_op(&mut memory).unwrap(), 2);
        assert_eq!(cpu.register_a, 0b_1010_1010);
        assert_eq!(cpu.status, StatusFlag::Negative as u8);

        assert_eq!(cpu.next_op(&mut memory).unwrap(), 3);
        assert_eq!(cpu.register_a, 0b_1000_0010);
        assert_eq!(cpu.status, StatusFlag::Negative as u8);

        assert_eq!(cpu.next_op(&mut memory).unwrap(), 4);
        assert_eq!(cpu.register_a, 0b_0000_0010);
        assert_eq!(cpu.status, 0);

        assert_eq!(cpu.next_op(&mut memory).unwrap(), 2);
        assert_eq!(cpu.register_a, 0b_0000_0000);
        assert_eq!(cpu.status, StatusFlag::Zero as u8);
    }

    #[test_case(op::CMP_IMM, (100, 0, 0); "compare register a")]
    #[test_case(op::CPX_IMM, (0, 100, 0); "compare register x")]
    #[test_case(op::CPY_IMM, (0, 0, 100); "compare register y")]
    fn compare_register(opcode: u8, register_values: (u8, u8, u8)) {
        let mut memory = [opcode, 100, opcode, 99, opcode, 101]; // 0x0C
        let mut cpu = Cpu::new();
        cpu.reset();
        (cpu.register_a, cpu.register_x, cpu.register_y) = register_values;

        assert_eq!(cpu.next_op(&mut memory).unwrap(), 2);
        assert_eq!(cpu.status, StatusFlag::Zero as u8 | StatusFlag::Carry as u8);

        assert_eq!(cpu.next_op(&mut memory).unwrap(), 2);
        assert_eq!(cpu.status, StatusFlag::Carry as u8);

        assert_eq!(cpu.next_op(&mut memory).unwrap(), 2);
        assert_eq!(cpu.status, StatusFlag::Negative as u8);
    }

    #[test]
    fn eor() {
        let mut memory = [
            op::EOR_IMM, 0b1100_1010,
            op::EOR_ABX, 0x0B, 0x00,
            op::EOR_ZER, 0x0C, 
            op::EOR_IMM, 0b0111_1101, op::NOP, op::NOP, 
            0b0101_1010,
            0b1010_1111
        ];
        let mut cpu = Cpu::new();
        cpu.register_a = 0b0100_0010;

        assert_eq!(cpu.next_op(&mut memory).unwrap(), 2);
        assert_eq!(cpu.register_a, 0b1000_1000);
        assert_eq!(cpu.status, StatusFlag::Negative as u8);

        assert_eq!(cpu.next_op(&mut memory).unwrap(), 4);
        assert_eq!(cpu.register_a, 0b1101_0010);
        assert_eq!(cpu.status, StatusFlag::Negative as u8);

        assert_eq!(cpu.next_op(&mut memory).unwrap(), 3);
        assert_eq!(cpu.register_a, 0b0111_1101);
        assert_eq!(cpu.status, 0);

        assert_eq!(cpu.next_op(&mut memory).unwrap(), 2);
        assert_eq!(cpu.register_a, 0);
        assert_eq!(cpu.status, StatusFlag::Zero as u8);
    }

}

#[cfg(test)]
mod jump_tests {
    use crate::cpu::{Cpu, op_codes as op};

    #[test]
    pub fn jump_indirect() {
        let mut memory = [
            op::JMP_IND, 0x05, 0x00,
            op::NOP, op::NOP,
            0xAB, 0xCD,
            op::NOP, op::NOP,
            op::NOP, op::NOP];
        let mut cpu = Cpu::new();

        assert_eq!(cpu.next_op(&mut memory).unwrap(), 5);
        assert_eq!(cpu.register_a, 0);
        assert_eq!(cpu.program_counter, 0xCDAB);
        assert_eq!(cpu.status, 0);
    }

    #[test]
    pub fn jump_absolute() {
        let mut memory = [
            op::JMP_ABS, 0x05, 0x00,
            op::NOP, op::NOP,
            0xAB, 0xCD,
            op::NOP, op::NOP,
            op::NOP, op::NOP];
        let mut cpu = Cpu::new();

        assert_eq!(cpu.next_op(&mut memory).unwrap(), 3);
        assert_eq!(cpu.register_a, 0);
        assert_eq!(cpu.program_counter, 0x0005);
        assert_eq!(cpu.status, 0);
    }
    
    #[test]
    pub fn jump_indirect_absolute() {
        let mut memory = [
            op::JMP_ABS, 0x05, 0x00,
            op::NOP, op::NOP,
            op::JMP_IND, 0x0C, 0x00,
            op::NOP, op::NOP,
            op::NOP, op::NOP,
            0xAB, 0xEF];
        let mut cpu = Cpu::new();

        assert_eq!(cpu.next_op(&mut memory).unwrap(), 3);
        assert_eq!(cpu.program_counter, 0x0005);
        assert_eq!(cpu.next_op(&mut memory).unwrap(), 5);
        assert_eq!(cpu.program_counter, 0xEFAB);
    }
}


#[cfg(test)]
mod address_modes_tests {
    use crate::cpu::{Cpu, op_codes as op, AddressMode, AddressResult};
    use test_case::test_case;

    #[test_case(0; "immediate_1")]
    #[test_case(1; "immediate_2")]
    fn test_modes(pc: u16) {
        let mem = [op::LDA_IMM, 0x05];
        let mut cpu = Cpu::new();
        cpu.program_counter = pc;
        assert_eq!(cpu.get_address(AddressMode::Immediate, &mem), pc.into());
    }
    
    #[test_case(0 => AddressResult::from(op::LDA_IMM as u16); "zero page 1")]
    #[test_case(1 => AddressResult::from(0x05u16); "zero page 2")]
    fn test_zero_page(pc: u16) -> AddressResult {
        let mem = [op::LDA_IMM, 0x05];
        let mut cpu = Cpu::new();
        cpu.program_counter = pc;
        cpu.get_address(AddressMode::ZeroPage, &mem)
    }


    #[test]
    fn test_zero_page_x() {
        let mem = [0x36, 0xF0];
        let mut cpu = Cpu::new();
        cpu.program_counter = 0;
        cpu.register_x = 0x15;
        cpu.register_y = 0x53;
        assert_eq!(cpu.get_address(AddressMode::ZeroPageX, &mem), 0x004B.into(), "add by x");
        assert_eq!(cpu.get_address(AddressMode::ZeroPageX, &mem), 0x0005.into(), "add by x overflow");
    }
    
    #[test]
    fn test_zero_page_y() {
        let mem = [0x36, 0xF0];
        let mut cpu = Cpu::new();
        cpu.program_counter = 0;
        cpu.register_x = 0x15;
        cpu.register_y = 0x53;
        assert_eq!(cpu.get_address(AddressMode::ZeroPageY, &mem), 0x0089.into(), "add by y");
        assert_eq!(cpu.get_address(AddressMode::ZeroPageY, &mem), 0x0043.into(), "add by y overflow");
    }

    
    #[test]
    fn test_absolute() {
        let mem = [0x36, 0xF0, 0xEF, 0xAB];
        let mut cpu = Cpu::new();
        cpu.program_counter = 0;
        cpu.register_x = 0x8E;
        cpu.register_y = 0x8F;
        assert_eq!(cpu.get_address(AddressMode::Absolute, &mem), 0xF036.into(), "absolute little endian 1");
        assert_eq!(cpu.get_address(AddressMode::Absolute, &mem), 0xABEF.into(), "absolute little endian 2");
    }

    
    #[test]
    fn test_absolute_x() {
        let mem = [0x36, 0xF0, 0xEF, 0xAB];
        let mut cpu = Cpu::new();
        cpu.program_counter = 0;
        cpu.register_x = 0x8E;
        cpu.register_y = 0x8F;
        assert_eq!(cpu.get_address(AddressMode::AbsoluteX, &mem), AddressResult { address: 0xF0C4, page_crossed: false}, "absolute little endian 1");
        assert_eq!(cpu.program_counter, 2);
        assert_eq!(cpu.get_address(AddressMode::AbsoluteX, &mem), AddressResult { address: 0xAC7D, page_crossed: true}, "absolute little endian 2");
        assert_eq!(cpu.program_counter, 4);
    }

    #[test]
    fn test_absolute_y() {
        let mem = [0x36, 0xF0, 0xEF, 0xAB];
        let mut cpu = Cpu::new();
        cpu.program_counter = 0;
        cpu.register_x = 0x8E;
        cpu.register_y = 0x8F;
        assert_eq!(cpu.get_address(AddressMode::AbsoluteY, &mem), AddressResult { address: 0xF0C5, page_crossed: false}, "absolute little endian 1");
        assert_eq!(cpu.program_counter, 2);
        assert_eq!(cpu.get_address(AddressMode::AbsoluteY, &mem), AddressResult { address: 0xAC7E, page_crossed: true}, "absolute little endian 2");
        assert_eq!(cpu.program_counter, 4);
    }

    #[test]
    fn test_indirect() {
        let mem = [0x08, 0x00, op::NOP, op::NOP, op::NOP, op::NOP, op::NOP, op::NOP, 0xAB, 0xCD];
        let mut cpu = Cpu::new();
        assert_eq!(cpu.get_address(AddressMode::Indirect, &mem), AddressResult { address: 0xCDAB, page_crossed: false});
        assert_eq!(cpu.program_counter, 2);
    }

    
    #[test]
    fn test_indirect_page_wrap_around() {
        let mut mem = [0; 0x0200];
        mem[0x0000] = 0x01;
        mem[0x00FF] = 0x05;
        mem[0x0100] = 0xAB;
        mem[0x0105] = 0xAA;
        mem[0x0106] = 0xBB;
        let mut cpu = Cpu::new();
        cpu.program_counter = 0x00FF;

        // With the page wrapping bug of indirect, the indirect address should be 0x0105, and not 0xAB05 (this would panic)
        // At 0x0105, we will get 0xBBAA

        assert_eq!(cpu.get_address(AddressMode::Indirect, &mem), 0xBBAA.into());
        assert_eq!(cpu.program_counter, 0x0101);
    }
}

#[cfg(test)]
mod address_result_tests {
    use crate::cpu::AddressResult;
    
    #[test]
    fn into() {
        let address: u16 = 0x12;
        let result: AddressResult = address.into();
        assert_eq!(result.address, address);
        assert_eq!(result.page_crossed, false);
    }
}

#[cfg(test)]
impl<const N: usize> Bus<u16> for [u8; N] {
    fn read_byte(&self, address: u16) -> u8 {
        self[address as usize]
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        self[address as usize] = value;
    }
}