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

#[derive(Clone, Copy)]
enum AddressMode {
    Implied,
    Immediate,

    ZeroPage, ZeroPageX, ZeroPageY,
    Absolute, AbsoluteX, AbsoluteY,

    IndirectX, IndirectY
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
enum InstructionCode {
    ADC, AND, ASL, BCC, BCS, BEQ, BIT, BMI, BNE, BPL, BRK, BVC, BVS, CLC,
    CLD, CLI, CLV, CMP, CPX, CPY, DEC, DEX, DEY, EOR, INC, INX, INY, JMP,
    JSR, LDA, LDX, LDY, LSR, NOP, ORA, PHA, PHP, PLA, PLP, ROL, ROR, RTI,
    RTS, SBC, SEC, SED, SEI, STA, STX, STY, TAX, TAY, TSX, TXA, TXS, TYA
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
struct Instruction {
    address_mode: AddressMode,
    instruction: InstructionCode, 
    bytes: u8,
    cycles: u8,
    can_cross_page: bool
}

impl Instruction {
    pub const fn new(mode: AddressMode, instr: InstructionCode, bytes: u8, cycles: u8, cross_page: bool) -> Self {
        Self {
            address_mode: mode,
            instruction: instr,
            bytes: bytes,
            cycles: cycles,
            can_cross_page: cross_page
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

}

static INSTRUCTION_SET: [Option<Instruction>; 0xFF] = {
    let mut table: [Option<Instruction>; 0xFF] = [None; 0xFF];

    macro_rules! add_instruction {
        ($code:ident, $addr:ident, $op:ident, $bytes:expr, $cycles:expr, $cross:expr) => {
            table[op_codes::$op as usize] = Some(Instruction::new(AddressMode::$addr, InstructionCode::$code, $bytes, $cycles, $cross));
        };
    }

    add_instruction!(BRK, Implied, BRK, 2, 7, false);
    add_instruction!(NOP, Implied, NOP, 1, 2, false);
    add_instruction!(CLC, Implied, CLC, 1, 2, false);
    add_instruction!(CLD, Implied, CLD, 1, 2, false);
    add_instruction!(CLI, Implied, CLI, 1, 2, false);
    add_instruction!(CLV, Implied, CLV, 1, 2, false);

    add_instruction!(INX, Implied, INX, 1, 2, false);
    add_instruction!(INY, Implied, INY, 1, 2, false);

    add_instruction!(LDX, Immediate, LDX_IMM, 2, 2, false);
    add_instruction!(LDX, ZeroPage,  LDX_ZER, 2, 3, false);
    add_instruction!(LDX, ZeroPageY, LDX_ZEY, 2, 4, false);
    add_instruction!(LDX, Absolute,  LDX_ABS, 3, 4, false);
    add_instruction!(LDX, AbsoluteY, LDX_ABY, 3, 4, true);

    add_instruction!(LDY, Immediate, LDY_IMM, 2, 2, false);
    add_instruction!(LDY, ZeroPage,  LDY_ZER, 2, 3, false);
    add_instruction!(LDY, ZeroPageX, LDY_ZEX, 2, 4, false);
    add_instruction!(LDY, Absolute,  LDY_ABS, 3, 4, false);
    add_instruction!(LDY, AbsoluteX, LDY_ABX, 3, 4, true);

    add_instruction!(LDA, Immediate, LDA_IMM, 2, 2, false);
    add_instruction!(LDA, ZeroPage,  LDA_ZER, 2, 3, false);
    add_instruction!(LDA, ZeroPageX, LDA_ZEX, 2, 4, false);
    add_instruction!(LDA, Absolute,  LDA_ABS, 3, 4, false);
    add_instruction!(LDA, AbsoluteX, LDA_ABX, 3, 4,  true);
    add_instruction!(LDA, AbsoluteY, LDA_ABY, 3, 4,  true);
    add_instruction!(LDA, IndirectX, LDA_INX, 2, 6, false);
    add_instruction!(LDA, IndirectY, LDA_INY, 2, 5,  true);

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

    pub fn next_op(&mut self, bus: &impl Bus<u16>) -> Result<u8, String> {
        let next_instruction = bus.read_byte(self.program_counter);
        self.program_counter += 1;

        match &INSTRUCTION_SET[next_instruction as usize] {
            Some(instr) => {
                let address = self.get_address(instr.address_mode, bus);
                self.execute_op(instr.instruction, address, bus);
                return Ok(instr.cycles);
            },
            None => {
                return Err(format!("Operation {:02X} not supported", next_instruction))
            }
        }
    }

    fn execute_op(&mut self, instruction: InstructionCode, address: u16, bus: &impl Bus<u16>) {
        use InstructionCode::*;
        match instruction {
            ADC => { todo!(); }, 
            AND => { todo!(); }, 
            ASL => { todo!(); }, 
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
            CMP => { todo!(); }, 
            CPX => { todo!(); }, 
            CPY => { todo!(); }, 
            DEC => { todo!(); }, 
            DEX => { todo!(); }, 
            DEY => { todo!(); }, 
            EOR => { todo!(); }, 
            INC => { todo!(); }, 
            INX => { self.increment_register_x(); }, 
            INY => { self.increment_register_y(); }, 
            JMP => { todo!(); },
            JSR => { todo!(); }, 
            LDA => { self.register_a = self.load_value(address, bus); },
            LDX => { self.register_x = self.load_value(address, bus); },
            LDY => { self.register_y = self.load_value(address, bus); },
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
    }

    fn load_value(&mut self, address: u16, bus: &impl Bus<u16>) -> u8 {
        let value = bus.read_byte(address);
        self.set_status(StatusFlag::Negative, (value >> 7) == 1);
        self.set_status(StatusFlag::Zero, value == 0);
        return value;
    }

    fn get_address(&mut self, mode: AddressMode, bus: &impl Bus<u16>) -> u16 {
        match mode {
            AddressMode::Implied => {
                return 0
            },
            AddressMode::Immediate => {
                let address = self.program_counter;
                self.program_counter += 1;
                return address;
            },
            AddressMode::ZeroPage => {
                let address = bus.read_byte(self.program_counter);
                self.program_counter += 1;
                return address as u16;
            },
            AddressMode::ZeroPageX => {
                let mut address = bus.read_byte(self.program_counter) as u16;
                self.program_counter += 1;
                address += self.register_x as u16;
                return address & 0xFF; // Masked; 0x0080 + 0x00FF = 0x007F (and not 0x017F)
            }, 
            AddressMode::ZeroPageY => {
                let mut address = bus.read_byte(self.program_counter) as u16;
                self.program_counter += 1;
                address += self.register_y as u16;
                return address & 0xFF; // Masked; 0x0080 + 0x00FF = 0x007F (and not 0x017F)
            }
            _ => { todo!(); }
        }
    }

    fn set_status(&mut self, key: StatusFlag, value: bool) {
        if value {
            self.status |= key as u8;
        } else {
            self.status &= !(key as u8)
        }
    }
    
    fn increment_register_x(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
        self.set_status(StatusFlag::Negative, self.register_x >> 7 == 1);
        self.set_status(StatusFlag::Zero, self.register_x == 0);
    }

    fn increment_register_y(&mut self) {
        self.register_y = self.register_y.wrapping_add(1);
        self.set_status(StatusFlag::Negative, self.register_y >> 7 == 1);
        self.set_status(StatusFlag::Zero, self.register_y == 0);
    }
}

#[cfg(test)]
mod load_register_tests {
    use crate::cpu::Cpu;
    use crate::cpu::op_codes::*;

    macro_rules! test_load_immediate {
        ($op:ident, $register:ident) => {
            #[allow(non_snake_case)]
            mod $op {
                use crate::cpu::{Cpu, StatusFlag};
                use crate::cpu::op_codes::*;

                #[test]
                fn immediate() {
                    let mem = [$op, 0x12];
                    let mut cpu = Cpu::new();
                    assert_eq!(cpu.next_op(&mem).unwrap(), 2);
                    assert_eq!(cpu.$register, 0x12);
                    assert_eq!(cpu.program_counter, 2);
                    assert_eq!(cpu.status, 0);
                }

                #[test]
                fn immediate_negative_status() {
                    let mem = [$op, 0x85];
                    let mut cpu = Cpu::new();
                    assert_eq!(cpu.next_op(&mem).unwrap(), 2);
                    assert_eq!(cpu.$register, 0x85);
                    assert_eq!(cpu.program_counter, 2);
                    assert_eq!(cpu.status, StatusFlag::Negative as u8);
                }

                #[test]
                fn immediate_zero_status() {
                    let mem = [$op, 0x00];
                    let mut cpu = Cpu::new();
                    assert_eq!(cpu.next_op(&mem).unwrap(), 2);
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
        let mem = [LDA_ZER, 0x05, NOP, NOP, BRK, 0x33];
        let mut cpu = Cpu::new();
        assert_eq!(cpu.next_op(&mem).unwrap(), 3);
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
        
        let mem = [op::INX, op::INX, op::INX];
        let mut cpu = Cpu::new();
        cpu.reset();
        cpu.register_a = 0xEE;
        cpu.register_x = 0xFE;
        cpu.register_y = 0xDF;


        assert_eq!(cpu.next_op(&mem).unwrap(), 2);        
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y), (0xEE, 0xFF, 0xDF));
        assert_eq!(cpu.status, StatusFlag::Negative as u8);

        assert_eq!(cpu.next_op(&mem).unwrap(), 2);
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y), (0xEE, 0x00, 0xDF));
        assert_eq!(cpu.status, StatusFlag::Zero as u8);

        assert_eq!(cpu.next_op(&mem).unwrap(), 2);
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y), (0xEE, 0x01, 0xDF));
        assert_eq!(cpu.status, 0);
    }

    
    #[test]
    fn increment_register_y() {
        
        let mem = [op::INY, op::INY, op::INY];
        let mut cpu = Cpu::new();
        cpu.reset();
        cpu.register_a = 0xEE;
        cpu.register_x = 0xAB;
        cpu.register_y = 0xFE;


        assert_eq!(cpu.next_op(&mem).unwrap(), 2);        
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y), (0xEE, 0xAB, 0xFF));
        assert_eq!(cpu.status, StatusFlag::Negative as u8);

        assert_eq!(cpu.next_op(&mem).unwrap(), 2);
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y), (0xEE, 0xAB, 0x00));
        assert_eq!(cpu.status, StatusFlag::Zero as u8);

        assert_eq!(cpu.next_op(&mem).unwrap(), 2);
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y), (0xEE, 0xAB, 0x01));
        assert_eq!(cpu.status, 0);
    }
}

#[cfg(test)]
mod direct_instruction_tests {
    use crate::cpu::{Cpu, StatusFlag, op_codes::{self as op}};
    use test_case::test_case;

    #[test]
    fn break_set_status() {
        let mem = [op::LDA_IMM, 0x00, op::BRK];
        let mut cpu = Cpu::new();

        cpu.next_op(&mem).unwrap();
        assert_eq!(cpu.has_breaked(), false);
        
        cpu.next_op(&mem).unwrap();
        assert_eq!(cpu.has_breaked(), true);
    }

    #[test_case(op::CLC, StatusFlag::Carry; "clear carry bit")] 
    #[test_case(op::CLD, StatusFlag::Decimal; "clear decimal bit")] 
    #[test_case(op::CLI, StatusFlag::Interrupt; "clear interrupt bit")] 
    #[test_case(op::CLV, StatusFlag::Overflow; "clear overflow bit")] 
    fn clear_codes(op_code: u8, status_bit: StatusFlag) {
        let mem = [op_code];
        let mut cpu = Cpu::new();
        let status_flag = status_bit as u8;

        cpu.reset();
        assert_eq!(cpu.next_op(&mem).unwrap(), 2);
        assert_eq!(cpu.status, 0);

        cpu.reset();
        cpu.status = status_flag;
        assert_eq!(cpu.next_op(&mem).unwrap(), 2);
        assert_eq!(cpu.status, 0);

        cpu.reset();
        cpu.status = 0xFF;
        assert_eq!(cpu.next_op(&mem).unwrap(), 2);
        assert_eq!(cpu.status, !status_flag);
    }

    #[test]
    fn nop_code() {
        const N: usize = 6;
        let mem: [u8; N] = [op::NOP, op::NOP, op::NOP, op::NOP, op::NOP, op::NOP ];
        let mut cpu = Cpu::new();
        let mut cycles = 0u8;
        cpu.reset();
        
        for _ in 0..N {
            cycles += cpu.next_op(&mem).unwrap();
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
mod address_modes_tests {
    use crate::cpu::{Cpu, op_codes as op, AddressMode};
    use test_case::test_case;

    #[test_case(0; "immediate_1")]
    #[test_case(1; "immediate_2")]
    fn test_modes(pc: u16) {
        let mem = [op::LDA_IMM, 0x05];
        let mut cpu = Cpu::new();
        cpu.program_counter = pc;
        assert_eq!(cpu.get_address(AddressMode::Immediate, &mem), pc);
    }

    
    #[test_case(0 => op::LDA_IMM as u16; "zero page 1")]
    #[test_case(1 => 0x05; "zero page 2")]
    fn test_zero_page(pc: u16) -> u16 {
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
        assert_eq!(cpu.get_address(AddressMode::ZeroPageX, &mem), 0x004B, "add by x");
        assert_eq!(cpu.get_address(AddressMode::ZeroPageX, &mem), 0x0005, "add by x overflow");
    }
    
    #[test]
    fn test_zero_page_y() {
        let mem = [0x36, 0xF0];
        let mut cpu = Cpu::new();
        cpu.program_counter = 0;
        cpu.register_x = 0x15;
        cpu.register_y = 0x53;
        assert_eq!(cpu.get_address(AddressMode::ZeroPageY, &mem), 0x0089, "add by y");
        assert_eq!(cpu.get_address(AddressMode::ZeroPageY, &mem), 0x0043, "add by y overflow");
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