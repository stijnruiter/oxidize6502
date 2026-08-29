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

    IndirectX, IndirectY
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
enum Mnemonic {
    ADC, AND, ASL, BCC, BCS, BEQ, BIT, BMI, BNE, BPL, BRK, BVC, BVS, CLC,
    CLD, CLI, CLV, CMP, CPX, CPY, DEC, DEX, DEY, EOR, INC, INX, INY, JMP,
    JSR, LDA, LDX, LDY, LSR, NOP, ORA, PHA, PHP, PLA, PLP, ROL, ROR, RTI,
    RTS, SBC, SEC, SED, SEI, STA, STX, STY, TAX, TAY, TSX, TXA, TXS, TYA
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
struct Instruction {
    address_mode: AddressMode,
    mnemonic: Mnemonic,
    cycles: u8,
    can_cross_page: bool
}

impl Instruction {
    pub const fn new(mode: AddressMode, mnemenic: Mnemonic, cycles: u8, cross_page: bool) -> Self {
        Self {
            address_mode: mode,
            mnemonic: mnemenic,
            cycles: cycles,
            can_cross_page: cross_page
        }
    }
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

pub mod op_codes {
    pub const BRK: u8 = 0x00;
    pub const NOP: u8 = 0xEA;
    pub const CLC: u8 = 0x18;
    pub const CLD: u8 = 0xD8;
    pub const CLI: u8 = 0x58;
    pub const CLV: u8 = 0xB8;

    pub const INC_ZER: u8 = 0xE6;
    pub const INC_ZEX: u8 = 0xF6;
    pub const INC_ABS: u8 = 0xEE;
    pub const INC_ABX: u8 = 0xFE;

    pub const INX: u8 = 0xE8;
    pub const INY: u8 = 0xC8;

    pub const LDX_IMM: u8 = 0xA2;
    pub const LDX_ZER: u8 = 0xA6;
    pub const LDX_ZEY: u8 = 0xB6;
    pub const LDX_ABS: u8 = 0xAE;
    pub const LDX_ABY: u8 = 0xBE;

    pub const LDY_IMM: u8 = 0xA0;
    pub const LDY_ZER: u8 = 0xA4;
    pub const LDY_ZEX: u8 = 0xB4;
    pub const LDY_ABS: u8 = 0xAC;
    pub const LDY_ABX: u8 = 0xBC;

    pub const LDA_IMM: u8 = 0xA9;
    pub const LDA_ZER: u8 = 0xA5;
    pub const LDA_ZEX: u8 = 0xB5;
    pub const LDA_ABS: u8 = 0xAD;
    pub const LDA_ABX: u8 = 0xBD;
    pub const LDA_ABY: u8 = 0xB9;
    pub const LDA_INX: u8 = 0xA1;
    pub const LDA_INY: u8 = 0xB1;

    pub const AND_IMM: u8 = 0x29;
    pub const AND_ZER: u8 = 0x25;
    pub const AND_ZEX: u8 = 0x35;
    pub const AND_ABS: u8 = 0x2D;
    pub const AND_ABX: u8 = 0x3D;
    pub const AND_ABY: u8 = 0x39;
    pub const AND_INX: u8 = 0x21;
    pub const AND_INY: u8 = 0x31;

    pub const ASL_ACC: u8 = 0x0A;
    pub const ASL_ZER: u8 = 0x06;
    pub const ASL_ZEX: u8 = 0x16;
    pub const ASL_ABS: u8 = 0x0E;
    pub const ASL_ABX: u8 = 0x1E;
    
    pub const CMP_IMM: u8 = 0xC9;
    pub const CMP_ZER: u8 = 0xC5;
    pub const CMP_ZEX: u8 = 0xD5;
    pub const CMP_ABS: u8 = 0xCD;
    pub const CMP_ABX: u8 = 0xDD;
    pub const CMP_ABY: u8 = 0xD9;
    pub const CMP_INX: u8 = 0xC1;
    pub const CMP_INY: u8 = 0xD1;

    pub const CPX_IMM: u8 = 0xE0;
    pub const CPX_ZER: u8 = 0xE4;
    pub const CPX_ABS: u8 = 0xEC;

    pub const CPY_IMM: u8 = 0xC0;
    pub const CPY_ZER: u8 = 0xC4;
    pub const CPY_ABS: u8 = 0xCC;

    pub const DEC_ZER: u8 = 0xC6;
    pub const DEC_ZEX: u8 = 0xD6;
    pub const DEC_ABS: u8 = 0xCE;
    pub const DEC_ABX: u8 = 0xDE;

    pub const DEX: u8 = 0xCA;
    pub const DEY: u8 = 0x88;

    pub const EOR_IMM: u8 = 0x49;
    pub const EOR_ZER: u8 = 0x45;
    pub const EOR_ZEX: u8 = 0x55;
    pub const EOR_ABS: u8 = 0x4D;
    pub const EOR_ABX: u8 = 0x5D;
    pub const EOR_ABY: u8 = 0x59;
    pub const EOR_INX: u8 = 0x41;
    pub const EOR_INY: u8 = 0x51;
}

static INSTRUCTION_SET: [Option<Instruction>; 0xFF] = {
    let mut table: [Option<Instruction>; 0xFF] = [None; 0xFF];

    macro_rules! add_instruction {
        ($code:ident, $addr:ident, $op:ident, $cycles:expr, $cross:expr) => {
            table[op_codes::$op as usize] = Some(Instruction::new(AddressMode::$addr, Mnemonic::$code, $cycles, $cross));
        };
    }

    add_instruction!(BRK, Implied, BRK, 7, false);
    add_instruction!(NOP, Implied, NOP, 2, false);
    add_instruction!(CLC, Implied, CLC, 2, false);
    add_instruction!(CLD, Implied, CLD, 2, false);
    add_instruction!(CLI, Implied, CLI, 2, false);
    add_instruction!(CLV, Implied, CLV, 2, false);

    add_instruction!(INC, ZeroPage,  INC_ZER, 5, false);
    add_instruction!(INC, ZeroPageX, INC_ZEX, 6, false);
    add_instruction!(INC, Absolute,  INC_ABS, 6, false);
    add_instruction!(INC, AbsoluteY, INC_ABX, 7, false);

    add_instruction!(INX, Implied, INX, 2, false);
    add_instruction!(INY, Implied, INY, 2, false);

    add_instruction!(LDX, Immediate, LDX_IMM, 2, false);
    add_instruction!(LDX, ZeroPage,  LDX_ZER, 3, false);
    add_instruction!(LDX, ZeroPageY, LDX_ZEY, 4, false);
    add_instruction!(LDX, Absolute,  LDX_ABS, 4, false);
    add_instruction!(LDX, AbsoluteY, LDX_ABY, 4, true);

    add_instruction!(LDY, Immediate, LDY_IMM, 2, false);
    add_instruction!(LDY, ZeroPage,  LDY_ZER, 3, false);
    add_instruction!(LDY, ZeroPageX, LDY_ZEX, 4, false);
    add_instruction!(LDY, Absolute,  LDY_ABS, 4, false);
    add_instruction!(LDY, AbsoluteX, LDY_ABX, 4, true);

    add_instruction!(LDA, Immediate, LDA_IMM, 2, false);
    add_instruction!(LDA, ZeroPage,  LDA_ZER, 3, false);
    add_instruction!(LDA, ZeroPageX, LDA_ZEX, 4, false);
    add_instruction!(LDA, Absolute,  LDA_ABS, 4, false);
    add_instruction!(LDA, AbsoluteX, LDA_ABX, 4,  true);
    add_instruction!(LDA, AbsoluteY, LDA_ABY, 4,  true);
    add_instruction!(LDA, IndirectX, LDA_INX, 6, false);
    add_instruction!(LDA, IndirectY, LDA_INY, 5,  true);

    add_instruction!(AND, Immediate, AND_IMM, 2, false);
    add_instruction!(AND, ZeroPage,  AND_ZER, 3, false); 
    add_instruction!(AND, ZeroPageX, AND_ZEX, 4, false);
    add_instruction!(AND, Absolute,  AND_ABS, 4, false);
    add_instruction!(AND, AbsoluteX, AND_ABX, 4, true); 
    add_instruction!(AND, AbsoluteY, AND_ABY, 4, true);
    add_instruction!(AND, IndirectX, AND_INX, 6, false); 
    add_instruction!(AND, IndirectY, AND_INY, 5, true);

    add_instruction!(ASL, Accumulator, ASL_ACC, 2, false);
    add_instruction!(ASL, ZeroPage,    ASL_ZER, 5, false);
    add_instruction!(ASL, ZeroPageX,   ASL_ZEX, 6, false);
    add_instruction!(ASL, Absolute,    ASL_ABS, 6, false);
    add_instruction!(ASL, AbsoluteX,   ASL_ABX, 7, false);

    add_instruction!(CMP, Immediate, CMP_IMM, 2, false); 
    add_instruction!(CMP, ZeroPage,  CMP_ZER, 3, false); 
    add_instruction!(CMP, ZeroPageX, CMP_ZEX, 4, false); 
    add_instruction!(CMP, Absolute,  CMP_ABS, 4, false); 
    add_instruction!(CMP, AbsoluteX, CMP_ABX, 4, true);
    add_instruction!(CMP, AbsoluteY, CMP_ABY, 4, true);
    add_instruction!(CMP, IndirectX, CMP_INX, 6, false);
    add_instruction!(CMP, IndirectY, CMP_INY, 5, true);

    add_instruction!(CPX, Immediate, CPX_IMM, 2, false); 
    add_instruction!(CPX, ZeroPage,  CPX_ZER, 3, false); 
    add_instruction!(CPX, Absolute,  CPX_ABS, 4, false);

    add_instruction!(CPY, Immediate, CPY_IMM, 2, false); 
    add_instruction!(CPY, ZeroPage,	 CPY_ZER, 3, false); 
    add_instruction!(CPY, Absolute,	 CPY_ABS, 4, false);

    add_instruction!(DEC, ZeroPage,  DEC_ZER, 5, false); 
    add_instruction!(DEC, ZeroPageX, DEC_ZEX, 6, false); 
    add_instruction!(DEC, Absolute,  DEC_ABS, 6, false); 
    add_instruction!(DEC, AbsoluteX, DEC_ABX, 7, false);
    
    add_instruction!(DEX, Implied, DEX, 2, false);
    add_instruction!(DEY, Implied, DEY, 2, false);


    add_instruction!(EOR, Immediate, EOR_IMM, 2, false);
    add_instruction!(EOR, ZeroPage,  EOR_ZER, 3, false);
    add_instruction!(EOR, ZeroPageX, EOR_ZEX, 4, false);
    add_instruction!(EOR, Absolute,  EOR_ABS, 4, false);
    add_instruction!(EOR, AbsoluteX, EOR_ABX, 4, true);
    add_instruction!(EOR, AbsoluteY, EOR_ABY, 4, true);
    add_instruction!(EOR, IndirectX, EOR_INX, 6, false);
    add_instruction!(EOR, IndirectY, EOR_INY, 5, true);


    table
};

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
            JMP => { todo!(); },
            JSR => { todo!(); }, 
            LDA => { self.register_a = self.load_value(address_result.address, bus); },
            LDX => { self.register_x = self.load_value(address_result.address, bus); },
            LDY => { self.register_y = self.load_value(address_result.address, bus); },
            LSR => { todo!(); }, 
            NOP => { /* do nothing */ }
            ORA => { todo!(); }, 
            PHA => { todo!(); }, 
            PHP => { todo!(); }, 
            PLA => { todo!(); }, 
            PLP => { todo!(); }, 
            ROL => { todo!(); }, 
            ROR => { todo!(); }, 
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
            AddressMode::Absolute => { self.read_word_little_endian(bus).into() },
            AddressMode::AbsoluteX => {
                let address = self.read_word_little_endian(bus);
                let address_offset_x = address.wrapping_add(self.register_x as u16);
                AddressResult {
                    address: address_offset_x,
                    page_crossed: address & 0xFF00 != address_offset_x & 0xFF00
                }
            },
            AddressMode::AbsoluteY => {
                let address = self.read_word_little_endian(bus);
                let address_offset_y = address.wrapping_add(self.register_y as u16);
                AddressResult {
                    address: address_offset_y,
                    page_crossed: address & 0xFF00 != address_offset_y & 0xFF00
                }
            }
            _ => { todo!(); }
        }
    }

    fn read_word_little_endian(&mut self, bus: &impl Bus<u16>) -> u16 {
        let low = bus.read_byte(self.program_counter) as u16;
        self.program_counter += 1;
        let high = bus.read_byte(self.program_counter) as u16;
        self.program_counter += 1;
        
        high << 8 | low
    }

    fn set_status(&mut self, key: StatusFlag, value: bool) {
        if value {
            self.status |= key as u8;
        } else {
            self.status &= !(key as u8)
        }
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

    #[test]
    fn asl_accumulator() {
        let mut memory = [op::ASL_ACC, op::ASL_ACC, op::ASL_ACC, op::ASL_ACC, op::ASL_ACC, op::ASL_ACC];
        let mut cpu = Cpu::new();
        cpu.reset();
        cpu.register_a = 0b1100_1000;
        
        assert_eq!(cpu.next_op(&mut memory).unwrap(), 2);
        assert_eq!(cpu.register_a, 0b1001_0000);
        assert_eq!(cpu.program_counter, 1);
        assert_eq!(cpu.status, StatusFlag::Carry as u8 | StatusFlag::Negative as u8);

        assert_eq!(cpu.next_op(&mut memory).unwrap(), 2);
        assert_eq!(cpu.register_a, 0b0010_0000);
        assert_eq!(cpu.program_counter, 2);
        assert_eq!(cpu.status, StatusFlag::Carry as u8);

        assert_eq!(cpu.next_op(&mut memory).unwrap(), 2);
        assert_eq!(cpu.register_a, 0b0100_0000);
        assert_eq!(cpu.program_counter, 3);
        assert_eq!(cpu.status, 0);    
        
        assert_eq!(cpu.next_op(&mut memory).unwrap(), 2);
        assert_eq!(cpu.register_a, 0b1000_0000);
        assert_eq!(cpu.program_counter, 4);
        assert_eq!(cpu.status, StatusFlag::Negative as u8);   
        
        assert_eq!(cpu.next_op(&mut memory).unwrap(), 2);
        assert_eq!(cpu.register_a, 0b0000_0000);
        assert_eq!(cpu.program_counter, 5);
        assert_eq!(cpu.status, StatusFlag::Carry as u8 | StatusFlag::Zero as u8);
    }
    
    #[test]
    fn asl_zero_page() {
        let mut memory = [
            op::ASL_ZER, 0x0B, op::ASL_ZER, 0x0B, op::ASL_ZER, 0x0B, op::ASL_ZER, 0x0B, 
            op::ASL_ZER, 0x0B, op::NOP, 0b1100_1000];
        let mut cpu = Cpu::new();
        cpu.reset();
        
        assert_eq!(cpu.next_op(&mut memory).unwrap(), 5);
        assert_eq!(memory[0x0B], 0b1001_0000);
        assert_eq!(cpu.program_counter, 2);
        assert_eq!(cpu.status, StatusFlag::Carry as u8 | StatusFlag::Negative as u8);

        assert_eq!(cpu.next_op(&mut memory).unwrap(), 5);
        assert_eq!(memory[0x0B], 0b0010_0000);
        assert_eq!(cpu.program_counter, 4);
        assert_eq!(cpu.status, StatusFlag::Carry as u8);

        assert_eq!(cpu.next_op(&mut memory).unwrap(), 5);
        assert_eq!(memory[0x0B], 0b0100_0000);
        assert_eq!(cpu.program_counter, 6);
        assert_eq!(cpu.status, 0);    
        
        assert_eq!(cpu.next_op(&mut memory).unwrap(), 5);
        assert_eq!(memory[0x0B], 0b1000_0000);
        assert_eq!(cpu.program_counter, 8);
        assert_eq!(cpu.status, StatusFlag::Negative as u8);   
        
        assert_eq!(cpu.next_op(&mut memory).unwrap(), 5);
        assert_eq!(memory[0x0B], 0b0000_0000);
        assert_eq!(cpu.program_counter, 10);
        assert_eq!(cpu.status, StatusFlag::Carry as u8 | StatusFlag::Zero as u8);

        assert_eq!(cpu.register_a, 0);
    }

    #[test]
    fn logical_and() {
        let mut memory = [
            op::AND_IMM, 0b_1010_1010, op::AND_ZER, 0x0B, op::AND_ABS, 0x0C, 0x00, 
            op::AND_IMM, 0b_1111_0000, op::NOP, op::NOP, 
            0b_1001_0110, // 0x0B
            0b_0000_1111,]; // 0x0C
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
        assert_eq!(cpu.get_address(AddressMode::AbsoluteX, &mem), AddressResult { address: 0xAC7D, page_crossed: true}, "absolute little endian 2");
    }

    #[test]
    fn test_absolute_y() {
        let mem = [0x36, 0xF0, 0xEF, 0xAB];
        let mut cpu = Cpu::new();
        cpu.program_counter = 0;
        cpu.register_x = 0x8E;
        cpu.register_y = 0x8F;
        assert_eq!(cpu.get_address(AddressMode::AbsoluteY, &mem), AddressResult { address: 0xF0C5, page_crossed: false}, "absolute little endian 1");
        assert_eq!(cpu.get_address(AddressMode::AbsoluteY, &mem), AddressResult { address: 0xAC7E, page_crossed: true}, "absolute little endian 2");
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