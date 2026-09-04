/**
 * https://6502.org/users/obelisk/6502/reference.html
 */

 use crate::{address::{AddressMode}, bus::Bus, instructions::{INSTRUCTION_SET, Instruction}};

use bitflags::bitflags;

bitflags! {
    #[derive(Clone, Copy, PartialEq, Debug)]
    pub struct StatusFlag: u8 {
        const Negative =  0x80;
        const Overflow =  0x40;
        const Unused =    0x20;
        const Break =     0x10;
        const Decimal =   0x08;
        const InterruptDisable = 0x04;
        const Zero =      0x02;
        const Carry =     0x01;
    }
}

pub struct Cpu {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,

    pub program_counter: u16,
    pub stack_pointer: u8,

    pub status: StatusFlag
}

impl Cpu {
    pub fn new() -> Self {
        Self { 
            register_a: 0, 
            register_x: 0, 
            register_y: 0, 
            
            program_counter: 0, 
            stack_pointer: 0, 
            
            status: StatusFlag::empty()
        }
    }

    pub fn reset(&mut self) {
        self.register_a =  0;
        self.register_x =  0; 
        self.register_y =  0; 
        
        self.program_counter = 0; 
        self.stack_pointer = 0xFD; 
        self.status = StatusFlag::empty();
    }

    pub fn run_step(&mut self, bus: &mut impl Bus<u16>) -> Result<u8, String> {
        let next_instruction = bus.read_byte(self.program_counter);
        self.program_counter += 1;

        match &INSTRUCTION_SET[next_instruction as usize] {
            Some(instruction) => {
                Ok(self.execute_instruction(instruction, bus))
            },
            None => {
                Err(format!("Operation {:02X} not supported", next_instruction))
            }
        }
    }

    fn execute_instruction(&mut self, instruction: &Instruction, bus: &mut impl Bus<u16>) -> u8 {
        use crate::instructions::Mnemonic::*;
        let mut branch_taken: bool = false;
        let address_result = instruction.address_mode.get_address(self, bus);
        match instruction.mnemonic {
            ADC => { 
                let a = self.register_a as u16;
                let m = bus.read_byte(address_result.address) as u16;
                let c = self.is_set(StatusFlag::Carry) as u16;
                let mut result = a + m + c;

                self.set_status(StatusFlag::Negative, result & 0x80 != 0);
                // Overflow occurs when signs before are equal, but not afterwards
                self.set_status(StatusFlag::Overflow, (!(a ^ m) & (a ^ result) & 0x80) != 0);
                self.set_status(StatusFlag::Zero, result & 0xFF == 0);

                if self.is_set(StatusFlag::Decimal) {
                    if (a & 0x0F) + (m & 0x0F) + c > 0x09 {
                        result += 0x06;
                    }

                    let carry = result > 0x99;
                    if carry {
                        result += 0x60;
                    }

                    self.set_status(StatusFlag::Carry, carry);
                }
                else {
                    self.set_status(StatusFlag::Carry, result > 0xFF);
                }

                self.register_a = result as u8;
            }, 
            AND => { 
                let value = bus.read_byte(address_result.address);
                self.register_a &= value;
                self.set_status(StatusFlag::Negative, self.register_a & 0x80 != 0);
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
            BCC => { 
                if !self.is_set(StatusFlag::Carry) {
                    self.program_counter = address_result.address;
                    branch_taken = true;
                }
            }, 
            BCS => {  
                if self.is_set(StatusFlag::Carry) {
                    self.program_counter = address_result.address;
                    branch_taken = true;
                }
            }, 
            BEQ => {
                if self.is_set(StatusFlag::Zero) {
                    self.program_counter = address_result.address;
                    branch_taken = true;
                }
            }, 
            BIT => { 
                let m = bus.read_byte(address_result.address);
                let value = self.register_a & m;
                self.set_status(StatusFlag::Zero, value == 0);
                self.set_status(StatusFlag::Overflow, (m >> 6) & 1 == 1);
                self.set_status(StatusFlag::Negative, m & 0x80 != 0);
            }, 
            BMI => {  
                if self.is_set(StatusFlag::Negative) {
                    self.program_counter = address_result.address;
                    branch_taken = true;
                }
            }, 
            BNE => {  
                if !self.is_set(StatusFlag::Zero) {
                    self.program_counter = address_result.address;
                    branch_taken = true;
                }
            }, 
            BPL => {  
                if !self.is_set(StatusFlag::Negative) {
                    self.program_counter = address_result.address;
                    branch_taken = true;
                }
            }, 
            BRK => {
                self.program_counter += 1; // Discard next byte
                self.push_stack(bus, (self.program_counter >> 8) as u8);
                self.push_stack(bus, (self.program_counter & 0xFF) as u8);
                let status = self.status | StatusFlag::Unused | StatusFlag::Break;
                self.push_stack(bus, status.bits());
                self.program_counter = bus.read_word_little_endian(0xFFFE);
                self.status |= StatusFlag::InterruptDisable;
            }
            BVC => {
                if !self.is_set(StatusFlag::Overflow) {
                    self.program_counter = address_result.address;
                    branch_taken = true;
                }
            }, 
            BVS => {
                if self.is_set(StatusFlag::Overflow) {
                    self.program_counter = address_result.address;
                    branch_taken = true;
                }
            }, 
            CLC => { self.set_status(StatusFlag::Carry, false); },
            CLD => { self.set_status(StatusFlag::Decimal, false); },
            CLI => { self.set_status(StatusFlag::InterruptDisable, false); },
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
                self.set_status(StatusFlag::Negative, result & 0x80 != 0);
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
            JSR => { 
                let pc = self.program_counter - 1;
                self.push_stack(bus, (pc >> 8) as u8);
                self.push_stack(bus, (pc & 0xFF) as u8);
                self.program_counter = address_result.address;
            }, 
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
                self.set_status(StatusFlag::Negative, self.register_a & 0x80 != 0);
             }, 
            PHA => { self.push_stack(bus, self.register_a); }, 
            PHP => { 
                let status = self.status | StatusFlag::Unused | StatusFlag::Break;
                self.push_stack(bus, status.bits()); }, 
            PLA => { 
                self.register_a = self.pull_stack(bus);
                self.set_status(StatusFlag::Zero, self.register_a == 0);
                self.set_status(StatusFlag::Negative, self.register_a & 0x80 != 0);
            }, 
            PLP => { 
                let status = StatusFlag::from_bits_retain(self.pull_stack(bus));
                // Discard Break, set unused high
                self.status = status & !StatusFlag::Break | StatusFlag::Unused;
            }, 
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
            RTI => {
                let status = StatusFlag::from_bits_retain(self.pull_stack(bus));
                let pc_low = self.pull_stack(bus) as u16;
                let pc_high = self.pull_stack(bus) as u16;
                self.status = (status & !StatusFlag::Break) | StatusFlag::Unused;
                self.program_counter = pc_high << 8 | pc_low;
            },
            RTS => { 
                let pc_low = self.pull_stack(bus) as u16;
                let pc_high = self.pull_stack(bus) as u16;
                self.program_counter = pc_high << 8 | pc_low;
                self.program_counter += 1;
            }, 
            SBC => { 
                let a = self.register_a as i16;
                let m = bus.read_byte(address_result.address) as i16;
                let c = self.is_set(StatusFlag::Carry) as i16;
                let binary_result = a - m - (1 -  c);

                self.set_status(StatusFlag::Negative, binary_result & 0x80 != 0);
                // Overflow occurs when signs before are different and also afterwards
                self.set_status(StatusFlag::Overflow, ((a ^ m) & (a ^ binary_result) & 0x80) != 0);
                self.set_status(StatusFlag::Zero, binary_result & 0xFF == 0);

                if self.is_set(StatusFlag::Decimal) {
                    let mut decimal_result = binary_result;

                    if (a & 0x0F) - (m & 0x0F) - (1 - c) < 0 {
                        decimal_result -= 0x06;
                    }

                    let no_borrow = binary_result >= 0;

                    if !no_borrow {
                        decimal_result -= 0x60;
                    }

                    self.set_status(StatusFlag::Carry, no_borrow);

                    self.register_a = decimal_result as u8;
                } else {
                    self.set_status(StatusFlag::Carry, binary_result >= 0);
                    self.register_a = binary_result as u8;
                }
            }, 
            SEC => { self.set_status(StatusFlag::Carry, true); }, 
            SED => { self.set_status(StatusFlag::Decimal, true); }, 
            SEI => { self.set_status(StatusFlag::InterruptDisable, true); }, 
            STA => { bus.write_byte(address_result.address, self.register_a); }, 
            STX => { bus.write_byte(address_result.address, self.register_x); }, 
            STY => { bus.write_byte(address_result.address, self.register_y); }, 
            TAX => { 
                self.register_x = self.register_a;
                self.set_status(StatusFlag::Zero, self.register_x == 0);
                self.set_status(StatusFlag::Negative, self.register_x & 0x80 != 0);
            }, 
            TAY => { 
                self.register_y = self.register_a;
                self.set_status(StatusFlag::Zero, self.register_y == 0);
                self.set_status(StatusFlag::Negative, self.register_y & 0x80 != 0);
            }, 
            TSX => { 
                self.register_x = self.stack_pointer;
                self.set_status(StatusFlag::Zero, self.register_x == 0);
                self.set_status(StatusFlag::Negative, self.register_x & 0x80 != 0);
            }, 
            TXA => { 
                self.register_a = self.register_x;
                self.set_status(StatusFlag::Zero, self.register_a == 0);
                self.set_status(StatusFlag::Negative, self.register_a & 0x80 != 0);
            }, 
            TXS => { self.stack_pointer = self.register_x; }, 
            TYA => { 
                self.register_a = self.register_y;
                self.set_status(StatusFlag::Zero, self.register_a == 0);
                self.set_status(StatusFlag::Negative, self.register_a & 0x80 != 0);
            }
        }

        if instruction.address_mode == AddressMode::Relative {
            if !branch_taken {
                return instruction.cycles;
            }
            if instruction.can_cross_page && address_result.page_crossed {
                return instruction.cycles + 2;
            }
            return instruction.cycles + 1;
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
        self.set_status(StatusFlag::Carry, value & 0x80 != 0);
        self.set_status(StatusFlag::Negative, new_value & 0x80 != 0);
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

        self.set_status(StatusFlag::Carry, value & 0x80 != 0);
        self.set_status(StatusFlag::Negative, new_value & 0x80 != 0);
        self.set_status(StatusFlag::Zero, new_value == 0);
        new_value
    }

    fn ror_value(&mut self, value: u8) -> u8 {
        let mut new_value = value >> 1;
        if self.is_set(StatusFlag::Carry) {
            new_value |= 1 << 7;
        }

        self.set_status(StatusFlag::Carry, value & 1 == 1);
        self.set_status(StatusFlag::Negative, new_value & 0x80 != 0);
        self.set_status(StatusFlag::Zero, new_value == 0);
        new_value
    }

    fn push_stack(&mut self, bus: &mut impl Bus<u16>, value: u8) {
        let stack_address = 0x0100u16 + self.stack_pointer as u16;
        bus.write_byte(stack_address, value);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    fn pull_stack(&mut self, bus: &mut impl Bus<u16>) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        let stack_address = 0x0100u16 + self.stack_pointer as u16;
        bus.read_byte(stack_address)
    }

    fn set_status(&mut self, key: StatusFlag, value: bool) {
        if value {
            self.status |= key;
        } else {
            self.status &= !key
        }
    }

    fn is_set(&self, key: StatusFlag) -> bool {
        self.status.contains(key)
    }
    
    fn increment_value(&mut self, value: u8) -> u8 {
        let increment_value = value.wrapping_add(1);
        self.set_status(StatusFlag::Negative, increment_value & 0x80 != 0);
        self.set_status(StatusFlag::Zero, increment_value == 0);
        return increment_value;
    }

    fn decrement_value(&mut self, value: u8) -> u8 {
        let decrement_value = value.wrapping_sub(1);
        self.set_status(StatusFlag::Negative, decrement_value & 0x80 != 0);
        self.set_status(StatusFlag::Zero, decrement_value == 0);
        return decrement_value;
    }
    
    fn compare(&mut self, register_value: u8, value: u8) {
        let result = register_value.wrapping_sub(value);
        self.set_status(StatusFlag::Carry, register_value >= value);
        self.set_status(StatusFlag::Zero, result == 0);
        self.set_status(StatusFlag::Negative, result & 0x80 != 0);
    }
}

#[cfg(test)]
mod load_register_tests {
    use crate::cpu::{Cpu, StatusFlag};
    use crate::instructions::op_codes::*;
    use test_case::test_case;
        
    #[test_case(LDA_IMM, 0x12, (0x12, 0xCD, 0xEF), StatusFlag::empty(); "load immediate accumulator")]
    #[test_case(LDA_IMM, 0x85, (0x85, 0xCD, 0xEF), StatusFlag::Negative; "load immediate accumulator negative")]
    #[test_case(LDA_IMM, 0x00, (0x00, 0xCD, 0xEF), StatusFlag::Zero; "load immediate accumulator zero")]
    
    #[test_case(LDX_IMM, 0x12, (0xAB, 0x12, 0xEF), StatusFlag::empty(); "load immediate register x")]
    #[test_case(LDX_IMM, 0x85, (0xAB, 0x85, 0xEF), StatusFlag::Negative; "load immediate register x negative")]
    #[test_case(LDX_IMM, 0x00, (0xAB, 0x00, 0xEF), StatusFlag::Zero; "load immediate register x zero")]

    #[test_case(LDY_IMM, 0x12, (0xAB, 0xCD, 0x12), StatusFlag::empty(); "load immediate register y")]
    #[test_case(LDY_IMM, 0x85, (0xAB, 0xCD, 0x85), StatusFlag::Negative; "load immediate register y negative")]
    #[test_case(LDY_IMM, 0x00, (0xAB, 0xCD, 0x00), StatusFlag::Zero; "load immediate register y zero")]
    fn immediate(op_code: u8, value_immediate: u8, expected_register_values: (u8, u8, u8), expected_status: StatusFlag) {
        let mut memory = [op_code, value_immediate];
        let mut cpu = Cpu::new();
        (cpu.register_a, cpu.register_x, cpu.register_y) = (0xAB, 0xCD, 0xEF);

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2);
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y), expected_register_values);
        assert_eq!(cpu.program_counter, 2);
        assert_eq!(cpu.status, expected_status);
    }

    #[test]
    fn lda_zero_page() {
        let mut mem = [LDA_ZER, 0x05, NOP, NOP, BRK, 0x33];
        let mut cpu = Cpu::new();
        
        assert_eq!(cpu.run_step(&mut mem).unwrap(), 3);
        assert_eq!(cpu.register_a, 0x33);
        assert_eq!(cpu.program_counter, 2);
        assert!(cpu.status.is_empty());
    }
}

#[cfg(test)]
mod store_register_tests {
    use crate::cpu::{Cpu};
    use crate::instructions::op_codes as op;
    use test_case::test_case;

    #[test_case(op::STA_ZER, 0x11, 3; "store register accumulator zero page")]
    #[test_case(op::STA_ABS, 0x11, 4; "store register accumulator absolute")]
    #[test_case(op::STX_ZER, 0xC0, 3; "store register x zero page")]
    #[test_case(op::STX_ABS, 0xC0, 4; "store register x absolute")]
    #[test_case(op::STY_ZER, 0xDE, 3; "store register y zero page")]
    #[test_case(op::STY_ABS, 0xDE, 4; "store register y absolute")]
    fn zero_page(instruction: u8, expected_value: u8, expected_cycles: u8){
        let register_values: (u8, u8, u8) = (0x11, 0xC0, 0xDE);
        let mut memory = [instruction, 0x05, 0x00,op::NOP, op::NOP, 0xAB];
        let mut cpu = Cpu::new();
        cpu.reset();
        (cpu.register_a, cpu.register_x, cpu.register_y) = register_values;
        assert_eq!(cpu.run_step(&mut memory).unwrap(), expected_cycles);
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y), register_values);
        assert_eq!(memory, [instruction, 0x05, 0x00,op::NOP, op::NOP, expected_value]);
    }
}

#[cfg(test)]
mod transfer_register_tests {
    use crate::cpu::{Cpu, StatusFlag};
    use crate::instructions::op_codes::*;
    use test_case::test_case;
    
    #[test_case(TAX, 0x5B, StatusFlag::empty(); "Transfer accumulator to X")]
    #[test_case(TAX, 0xFE, StatusFlag::Negative; "Transfer accumulator to X negative")]
    #[test_case(TAX, 0x00, StatusFlag::Zero; "Transfer accumulator to X zero")]
    fn transfer_accumulator_to_x(op_code: u8, accumulator: u8, expected_status: StatusFlag) {
        let mut memory = [op_code];
        let mut cpu = Cpu::new();
        cpu.reset();
        (cpu.register_a, cpu.register_x, cpu.register_y, cpu.stack_pointer) = (accumulator, 0x11, 0xCD, 0x1F);

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2, "Operation cycles");
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y, cpu.stack_pointer), (accumulator, accumulator, 0xCD, 0x1F), "CPU registers");
        assert_eq!(cpu.status, expected_status, "CPU status");
    }
    
    #[test_case(TAY, 0x5B, StatusFlag::empty(); "Transfer accumulator to Y")]
    #[test_case(TAY, 0xFE, StatusFlag::Negative; "Transfer accumulator to Y negative")]
    #[test_case(TAY, 0x00, StatusFlag::Zero; "Transfer accumulator to Y zero")]
    fn transfer_accumulator_to_y(op_code: u8, accumulator: u8, expected_status: StatusFlag) {
        let mut memory = [op_code];
        let mut cpu = Cpu::new();
        cpu.reset();
        (cpu.register_a, cpu.register_x, cpu.register_y, cpu.stack_pointer) = (accumulator, 0x11, 0xCD, 0x1F);

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2, "Operation cycles");
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y, cpu.stack_pointer), (accumulator, 0x11, accumulator, 0x1F), "CPU registers");
        assert_eq!(cpu.status, expected_status, "CPU status");
    }

    
    #[test_case(TXA, 0x5B, StatusFlag::empty(); "Transfer X to accumulator")]
    #[test_case(TXA, 0xFE, StatusFlag::Negative; "Transfer X to accumulator negative")]
    #[test_case(TXA, 0x00, StatusFlag::Zero; "Transfer X to accumulator zero")]
    fn transfer_x_to_accumulator(op_code: u8, register_x: u8, expected_status: StatusFlag) {
        let mut memory = [op_code];
        let mut cpu = Cpu::new();
        cpu.reset();
        (cpu.register_a, cpu.register_x, cpu.register_y, cpu.stack_pointer) = (0xAB, register_x, 0xCD, 0x1F);

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2, "Operation cycles");
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y, cpu.stack_pointer), (register_x, register_x, 0xCD, 0x1F), "CPU registers");
        assert_eq!(cpu.status, expected_status, "CPU status");
    }

    
    #[test_case(TYA, 0x5B, StatusFlag::empty(); "Transfer Y to accumulator")]
    #[test_case(TYA, 0xFE, StatusFlag::Negative; "Transfer Y to accumulator negative")]
    #[test_case(TYA, 0x00, StatusFlag::Zero; "Transfer Y to accumulator zero")]
    fn transfer_y_to_accumulator(op_code: u8, register_y: u8, expected_status: StatusFlag) {
        let mut memory = [op_code];
        let mut cpu = Cpu::new();
        cpu.reset();
        (cpu.register_a, cpu.register_x, cpu.register_y, cpu.stack_pointer) = (0xAB, 0xCD, register_y, 0x1F);

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2, "Operation cycles");
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y, cpu.stack_pointer), (register_y, 0xCD, register_y, 0x1F), "CPU registers");
        assert_eq!(cpu.status, expected_status, "CPU status");
    }

    #[test]
    fn transfer_x_to_stack() {
        let mut memory = [TXS, TXS];
        let mut cpu = Cpu::new();
        cpu.reset();
        cpu.status = StatusFlag::empty();
        (cpu.register_a, cpu.register_x, cpu.register_y, cpu.stack_pointer) = (0xAB, 0xFF, 0xCD, 0x1F);

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2);
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y, cpu.stack_pointer), (0xAB, 0xFF, 0xCD, 0xFF));
        assert!(cpu.status.is_empty());

        cpu.register_x = 0x00;
        cpu.status = StatusFlag::all();

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2);
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y, cpu.stack_pointer), (0xAB, 0x00, 0xCD, 0x00));
        assert!(cpu.status.is_all());
    }

    #[test_case(TSX, 0x5B, StatusFlag::empty(); "Transfer stack to X")]
    #[test_case(TSX, 0xFE, StatusFlag::Negative; "Transfer stack to X negative")]
    #[test_case(TSX, 0x00, StatusFlag::Zero; "Transfer stack to X zero")]
    fn transfer_stack_to_x(op_code: u8, stack_pointer: u8, expected_status: StatusFlag) {
        let mut memory = [op_code];
        let mut cpu = Cpu::new();
        cpu.reset();
        (cpu.register_a, cpu.register_x, cpu.register_y, cpu.stack_pointer) = (0xAB, 0x11, 0xCD, stack_pointer);

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2, "Operation cycles");
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y, cpu.stack_pointer), (0xAB, stack_pointer, 0xCD, stack_pointer), "CPU registers");
        assert_eq!(cpu.status, expected_status, "CPU status");
    }
}

#[cfg(test)]
mod increment_instruction_tests {
    use crate::cpu::{Cpu, StatusFlag};
    use crate::instructions::op_codes as op;

    #[test]
    fn increment_register_x() {
        
        let mut memory = [op::INX, op::INX, op::INX];
        let mut cpu = Cpu::new();
        cpu.reset();
        (cpu.register_a, cpu.register_x, cpu.register_y) = (0xEE, 0xFE, 0xDF);

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2);        
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y), (0xEE, 0xFF, 0xDF));
        assert_eq!(cpu.status, StatusFlag::Negative);

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2);
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y), (0xEE, 0x00, 0xDF));
        assert_eq!(cpu.status, StatusFlag::Zero);

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2);
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y), (0xEE, 0x01, 0xDF));
        assert!(cpu.status.is_empty());
    }

    
    #[test]
    fn decrement_register_x() {
        
        let mut memory = [op::DEX, op::DEX, op::DEX];
        let mut cpu = Cpu::new();
        cpu.reset();
        (cpu.register_a, cpu.register_x, cpu.register_y) = (0xEE, 0x01, 0xDF);

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2);        
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y), (0xEE, 0x00, 0xDF));
        assert_eq!(cpu.status, StatusFlag::Zero);

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2);
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y), (0xEE, 0xFF, 0xDF));
        assert_eq!(cpu.status, StatusFlag::Negative);
    }

    
    #[test]
    fn increment_register_y() {
        
        let mut memory = [op::INY, op::INY, op::INY];
        let mut cpu = Cpu::new();
        cpu.reset();
        (cpu.register_a, cpu.register_x, cpu.register_y) = (0xEE, 0xAB, 0xFE);

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2);        
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y), (0xEE, 0xAB, 0xFF));
        assert_eq!(cpu.status, StatusFlag::Negative);

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2);
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y), (0xEE, 0xAB, 0x00));
        assert_eq!(cpu.status, StatusFlag::Zero);

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2);
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y), (0xEE, 0xAB, 0x01));
        assert!(cpu.status.is_empty());
    }
    
    #[test]
    fn decrement_register_y() {
        
        let mut memory = [op::DEY, op::DEY, op::DEY];
        let mut cpu = Cpu::new();
        cpu.reset();
        (cpu.register_a, cpu.register_x, cpu.register_y) = (0xEE, 0xDF, 0x01);

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2);        
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y), (0xEE, 0xDF, 0x00));
        assert_eq!(cpu.status, StatusFlag::Zero);

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2);
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y), (0xEE, 0xDF, 0xFF));
        assert_eq!(cpu.status, StatusFlag::Negative);
    }

    #[test]
    fn increment_memory_zero_page() {
        let mut memory = [op::INC_ZER, 0x02, 0xFE];
        let mut cpu = Cpu::new();
        cpu.reset();

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 5);
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y), (0, 0, 0));
        assert_eq!(memory, [op::INC_ZER, 0x02, 0xFF]);
        assert_eq!(cpu.status, StatusFlag::Negative);

        cpu.reset();
        assert_eq!(cpu.run_step(&mut memory).unwrap(), 5);
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y), (0, 0, 0));
        assert_eq!(memory, [op::INC_ZER, 0x02, 0x00]);
        assert_eq!(cpu.status, StatusFlag::Zero);
        
        cpu.reset();
        assert_eq!(cpu.run_step(&mut memory).unwrap(), 5);
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y), (0, 0, 0));
        assert_eq!(memory, [op::INC_ZER, 0x02, 0x01]);
        assert!(cpu.status.is_empty());
    }

    #[test]
    fn increment_memory_zero_page_x() {
        let mut memory = [op::INC_ZEX, 0x02, op::NOP, op::NOP, 0xFE, op::NOP];
        let mut cpu = Cpu::new();
        cpu.reset();
        cpu.register_x = 0x02;

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 6);
        assert_eq!(memory, [op::INC_ZEX, 0x02, op::NOP, op::NOP, 0xFF, op::NOP]);
        assert_eq!(cpu.status, StatusFlag::Negative);
    }

    #[test]
    fn decrement_memory_zero_page() {
        let mut memory = [op::DEC_ZER, 0x02, 0x01];
        let mut cpu = Cpu::new();
        cpu.reset();

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 5);
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y), (0, 0, 0));
        assert_eq!(memory, [op::DEC_ZER, 0x02, 0x00]);
        assert_eq!(cpu.status,  StatusFlag::Zero);

        cpu.reset();
        assert_eq!(cpu.run_step(&mut memory).unwrap(), 5);
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y), (0, 0, 0));
        assert_eq!(memory, [op::DEC_ZER, 0x02, 0xFF]);
        assert_eq!(cpu.status, StatusFlag::Negative);
    }
    
    #[test]
    fn decrement_memory_zero_page_x() {
        let mut memory = [op::DEC_ZEX, 0x02, op::NOP, op::NOP, 0xFE, op::NOP];
        let mut cpu = Cpu::new();
        cpu.reset();
        cpu.register_x = 0x02;

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 6);
        assert_eq!(memory, [op::DEC_ZEX, 0x02, op::NOP, op::NOP, 0xFD, op::NOP]);
        assert_eq!(cpu.status, StatusFlag::Negative);
    }
}

#[cfg(test)]
mod direct_instruction_tests {
    use crate::cpu::{Cpu, StatusFlag};
    use crate::instructions::op_codes as op;
    use test_case::test_case;


    #[test_case(op::CLC, StatusFlag::Carry; "clear carry bit")] 
    #[test_case(op::CLD, StatusFlag::Decimal; "clear decimal bit")] 
    #[test_case(op::CLI, StatusFlag::InterruptDisable; "clear interrupt bit")] 
    #[test_case(op::CLV, StatusFlag::Overflow; "clear overflow bit")] 
    fn clear_codes(op_code: u8, status_bit: StatusFlag) {
        let mut memory = [op_code];
        let mut cpu = Cpu::new();
        cpu.reset();

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2);
        assert!(cpu.status.is_empty());

        cpu.reset();
        cpu.status = status_bit;
        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2);
        assert!(cpu.status.is_empty());

        cpu.reset();
        cpu.status = StatusFlag::all();
        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2);
        assert_eq!(cpu.status, !status_bit);
    }

    
    #[test_case(op::SEC, StatusFlag::Carry; "set carry bit")]
    #[test_case(op::SEI, StatusFlag::InterruptDisable; "set interrupt bit")] 
    fn set_codes(op_code: u8, status_bit: StatusFlag) {
        let mut memory = [op_code];
        let mut cpu = Cpu::new();
        cpu.reset();
        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2);
        assert_eq!(cpu.status, status_bit);

        cpu.reset();
        cpu.status = status_bit;
        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2);
        assert_eq!(cpu.status, status_bit);

        cpu.reset();
        cpu.status = StatusFlag::all();
        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2);
        assert!(cpu.status.is_all());
    }

    #[test]
    fn set_decimal_should_panic() {
        let mut memory = [op::SED];
        let mut cpu = Cpu::new();

        cpu.reset();
        assert_eq!(cpu.is_set(StatusFlag::Decimal), false);

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2);
        assert_eq!(cpu.is_set(StatusFlag::Decimal), true);
    }

    #[test]
    fn nop_code() {
        const N: usize = 6;
        let mut mem: [u8; N] = [op::NOP, op::NOP, op::NOP, op::NOP, op::NOP, op::NOP ];
        let mut cpu = Cpu::new();
        let mut cycles = 0u8;
        cpu.reset();

        let previous_state = (cpu.register_a, cpu.register_x, cpu.register_y, cpu.stack_pointer, cpu.status);
        for _ in 0..N {
            cycles += cpu.run_step(&mut mem).unwrap();
        }
        assert_eq!(cycles, 2 * N as u8, "cycles executed");
        assert_eq!(cpu.program_counter, N as u16, "current program counter");
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y, cpu.stack_pointer, cpu.status), previous_state);
    }
}

#[cfg(test)]
mod operation_tests {
    use crate::cpu::{Cpu, StatusFlag};
    use crate::instructions::op_codes as op;
    use test_case::test_case;

    #[test_case(0b0001_1010,  StatusFlag::Zero | StatusFlag::Negative | StatusFlag::Overflow; "Bit test zero")]
    #[test_case(0b0101_0101, StatusFlag::Overflow | StatusFlag::Negative; "Bit test overflow")]
    #[test_case(0b1101_0101, StatusFlag::Overflow | StatusFlag::Negative; "Bit test overflow and negative")]
    #[test_case(0b1001_0101, StatusFlag::Negative | StatusFlag::Overflow; "Bit test negative")]
    #[test_case(0b0001_1111, StatusFlag::Negative | StatusFlag::Overflow; "Bit test nothing")]
    fn bit_test(accumulator: u8, expected_flags: StatusFlag) {
        let mut memory = [op::BIT_ZER, 0x02, 0b1110_0101];
        let mut cpu = Cpu::new();
        cpu.reset();

        (cpu.register_a, cpu.register_x, cpu.register_y, cpu.stack_pointer) = (accumulator, 0x11, 0xCD, 0x1F);
        assert_eq!(cpu.run_step(&mut memory).unwrap(), 3, "Operation cycles");
        assert_eq!((cpu.register_a, cpu.register_x, cpu.register_y, cpu.stack_pointer), (accumulator, 0x11, 0xCD, 0x1F), "Register values");
        assert_eq!(cpu.status, expected_flags, "CPU Flags");
    }

    #[test_case(0b1100_1000, 0b1001_0000, StatusFlag::Carry | StatusFlag::Negative; "asl zero page carry negative")]
    #[test_case(0b1001_0000, 0b0010_0000, StatusFlag::Carry; "asl zero page carry only")]
    #[test_case(0b0010_0000, 0b0100_0000, StatusFlag::empty(); "asl zer page regular")]
    #[test_case(0b0100_0000, 0b1000_0000, StatusFlag::Negative; "asl zero page negative only")]
    #[test_case(0b1000_0000, 0b0000_0000, StatusFlag::Carry |  StatusFlag::Zero; "asl zero page carry zero")]
    #[test_case(0b0000_0000, 0b0000_0000,  StatusFlag::Zero; "asl zero page zero only")]
    fn asl_accumulator(accumulator_before: u8, expected_after: u8, expected_status: StatusFlag) {
        let mut memory = [op::ASL_ACC];
        let mut cpu = Cpu::new();
        cpu.reset();
        cpu.register_a = accumulator_before;
        
        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2, "Operation cycles");
        assert_eq!(cpu.register_a, expected_after, "Accumulator");
        assert_eq!(cpu.program_counter, 1, "Program counter");
        assert_eq!(cpu.status, expected_status, "CPU status");
    }
    
    #[test_case(0b1100_1000, 0b1001_0000, StatusFlag::Carry | StatusFlag::Negative; "asl zero page carry negative")]
    #[test_case(0b1001_0000, 0b0010_0000, StatusFlag::Carry; "asl zero page carry only")]
    #[test_case(0b0010_0000, 0b0100_0000, StatusFlag::empty(); "asl zer page regular")]
    #[test_case(0b0100_0000, 0b1000_0000, StatusFlag::Negative; "asl zero page negative only")]
    #[test_case(0b1000_0000, 0b0000_0000, StatusFlag::Carry |  StatusFlag::Zero; "asl zero page carry zero")]
    #[test_case(0b0000_0000, 0b0000_0000,  StatusFlag::Zero; "asl zero page zero only")]
    fn asl_zero_page(before: u8, expected_after: u8, expected_status: StatusFlag) {
        let mut memory = [op::ASL_ZER, 0x02, before];
        let mut cpu = Cpu::new();
        cpu.reset();
        
        assert_eq!(cpu.run_step(&mut memory).unwrap(), 5, "Operation cycles");
        assert_eq!(memory[0x02], expected_after, "Memory value");
        assert_eq!(cpu.program_counter, 2, "Program counter");
        assert_eq!(cpu.status, expected_status, "CPU status");
        assert_eq!(cpu.register_a, 0, "Accumulator");
    }
    
    #[test_case(0b1100_1000, 0b0110_0100, StatusFlag::empty(); "Regular bit shift")]
    #[test_case(0b1100_1001, 0b0110_0100, StatusFlag::Carry; "Carry bit shift")]
    #[test_case(0b0000_0001, 0b0000_0000, StatusFlag::Carry |  StatusFlag::Zero; "Carry and zero")]
    #[test_case(0b0000_0000, 0b0000_0000,  StatusFlag::Zero; "Zero")]
    fn lsr_accumulator(before: u8, expected_after: u8, expected_status: StatusFlag) {
        let mut memory = [op::LSR_ACC];
        let mut cpu = Cpu::new();
        cpu.reset();
        cpu.set_status(StatusFlag::Negative, true);
        cpu.register_a = before;
        
        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2, "Operation cycles");
        assert_eq!(cpu.register_a, expected_after, "Accumulator");
        assert_eq!(cpu.program_counter, 1, "Program counter");
        assert_eq!(cpu.status, expected_status, "CPU status flags");
    }

    #[test_case(0b1100_1000, 0b0110_0100, StatusFlag::empty(); "Regular bit shift")]
    #[test_case(0b1100_1001, 0b0110_0100, StatusFlag::Carry; "Carry bit shift")]
    #[test_case(0b0000_0001, 0b0000_0000, StatusFlag::Carry |  StatusFlag::Zero; "Carry and zero")]
    #[test_case(0b0000_0000, 0b0000_0000, StatusFlag::Zero; "Zero")]
    fn lsr_zero_page(before: u8, expected_after: u8, expected_status: StatusFlag) {
        let mut memory = [op::LSR_ZER, 0x02, before];
        let mut cpu = Cpu::new();
        cpu.reset();
        cpu.set_status(StatusFlag::Negative, true);
        
        assert_eq!(cpu.run_step(&mut memory).unwrap(), 5, "Operation cycles");
        assert_eq!(memory[0x02], expected_after, "Memory value");
        assert_eq!(cpu.program_counter, 2, "Program counter");
        assert_eq!(cpu.status, expected_status, "CPU status");
        assert_eq!(cpu.register_a, 0, "");
    }

    // ORA, ROL, ROR
    #[test_case(0b1001_0110, 0b0000_0000, 0b1001_0110, StatusFlag::Negative; "unchanged with zero")]
    #[test_case(0b0000_0000, 0b1001_0110, 0b1001_0110, StatusFlag::Negative; "zero accumulator")]
    #[test_case(0b0110_1001, 0b0001_0110, 0b0111_1111, StatusFlag::empty(); "regular without status flags")]
    #[test_case(0, 0, 0,  StatusFlag::Zero; "zero")]
    fn ora(accumulator_before: u8, value_before: u8, expected_result: u8, expected_status: StatusFlag) {
        let mut memory = [op::ORA_IMM, value_before];
        let mut cpu = Cpu::new();
        cpu.reset();
        cpu.set_status(StatusFlag::Negative, true);
        cpu.register_a = accumulator_before;

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2, "Operation cycles");
        assert_eq!(cpu.register_a, expected_result, "Accumulator");
        assert_eq!(cpu.status, expected_status, "CPU status flags");
    }
    
    #[test_case(0b0000_0000, false, 0b0000_0000, StatusFlag::Zero; "unchanged")]
    #[test_case(0b0000_0000, true, 0b0000_0001, StatusFlag::empty(); "carry transferred to value")]
    #[test_case(0b1010_1010, true, 0b0101_0101, StatusFlag::Carry; "carry correctly set")]
    #[test_case(0b0000_1010, false, 0b0001_0100, StatusFlag::empty(); "flags cleared")]
    #[test_case(0b0100_1010, false, 0b1001_0100, StatusFlag::Negative; "negative is set")]
    fn rol_accumulator(accumulator_before: u8, carry_bit: bool, expected_value: u8, expected_status: StatusFlag) {
        let mut memory = [op::ROL_ACC];
        let mut cpu = Cpu::new();
        cpu.reset();
        cpu.set_status(StatusFlag::Negative, true);
        cpu.set_status(StatusFlag::Carry, carry_bit);
        cpu.register_a = accumulator_before;

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2, "Operation cycles");
        assert_eq!(cpu.register_a, expected_value, "Accumulator");
        assert_eq!(cpu.status, expected_status, "CPU status flags");
    }
    
    #[test_case(0b0000_0000, false, 0b0000_0000,  StatusFlag::Zero; "unchanged")]
    #[test_case(0b0000_0000, true, 0b0000_0001, StatusFlag::empty(); "carry transferred to value")]
    #[test_case(0b1010_1010, true, 0b0101_0101, StatusFlag::Carry; "carry correctly set")]
    #[test_case(0b0000_1010, false, 0b0001_0100, StatusFlag::empty(); "flags cleared")]
    #[test_case(0b0100_1010, false, 0b1001_0100, StatusFlag::Negative; "negative is set")]
    fn rol_memory(value_before: u8, carry_bit: bool, expected_value: u8, expected_status: StatusFlag) {
        let mut memory = [op::ROL_ZER, 0x02, value_before];
        let mut cpu = Cpu::new();
        cpu.reset();
        cpu.set_status(StatusFlag::Negative, true);
        cpu.set_status(StatusFlag::Carry, carry_bit);
        cpu.register_a = 0xAB;

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 5, "Operation cycles");
        assert_eq!(memory[2], expected_value, "Memory value");
        assert_eq!(cpu.status, expected_status, "CPU status flags");
        assert_eq!(cpu.register_a, 0xAB);
    }

    #[test_case(0b0000_0000, false, 0b0000_0000,  StatusFlag::Zero; "unchanged")]
    #[test_case(0b0000_0000, true, 0b1000_0000, StatusFlag::Negative; "carry transferred to value")]
    #[test_case(0b1010_1010, true, 0b1101_0101, StatusFlag::Negative; "negative correctly set, carry applied")]
    #[test_case(0b0000_1010, false, 0b0000_0101, StatusFlag::empty(); "flags cleared")]
    #[test_case(0b0100_1011, true, 0b1010_0101, StatusFlag::Carry | StatusFlag::Negative; "carry and negative are set")]
    #[test_case(0b0100_1011, false, 0b0010_0101, StatusFlag::Carry; "carry is set")]
    fn ror_accumulator(accumulator_before: u8, carry_bit: bool, expected_value: u8, expected_status: StatusFlag) {
        let mut memory = [op::ROR_ACC];
        let mut cpu = Cpu::new();
        cpu.reset();
        cpu.set_status(StatusFlag::Negative, true);
        cpu.set_status(StatusFlag::Carry, carry_bit);
        cpu.register_a = accumulator_before;

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2, "Operation cycles");
        assert_eq!(cpu.register_a, expected_value, "Accumulator");
        assert_eq!(cpu.status, expected_status, "CPU status flags");
    }

    #[test_case(0b0000_0000, false, 0b0000_0000,  StatusFlag::Zero; "unchanged")]
    #[test_case(0b0000_0000, true, 0b1000_0000, StatusFlag::Negative; "carry transferred to value")]
    #[test_case(0b1010_1010, true, 0b1101_0101, StatusFlag::Negative; "negative correctly set, carry applied")]
    #[test_case(0b0000_1010, false, 0b0000_0101, StatusFlag::empty(); "flags cleared")]
    #[test_case(0b0100_1011, true, 0b1010_0101, StatusFlag::Carry | StatusFlag::Negative; "carry and negative are set")]
    #[test_case(0b0100_1011, false, 0b0010_0101, StatusFlag::Carry; "carry is set")]
    fn ror_memory(memory_before: u8, carry_bit: bool, expected_value: u8, expected_status: StatusFlag) {
        let mut memory = [op::ROR_ZER, 0x02, memory_before];
        let mut cpu = Cpu::new();
        cpu.reset();
        cpu.set_status(StatusFlag::Negative, true);
        cpu.set_status(StatusFlag::Carry, carry_bit);
        cpu.register_a = 0xAB;

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 5, "Operation cycles");
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

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2);
        assert_eq!(cpu.register_a, 0b_1010_1010);
        assert_eq!(cpu.status, StatusFlag::Negative);

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 3);
        assert_eq!(cpu.register_a, 0b_1000_0010);
        assert_eq!(cpu.status, StatusFlag::Negative);

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 4);
        assert_eq!(cpu.register_a, 0b_0000_0010);
        assert!(cpu.status.is_empty());

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2);
        assert_eq!(cpu.register_a, 0b_0000_0000);
        assert_eq!(cpu.status,  StatusFlag::Zero);
    }

    #[test_case(op::CMP_IMM, (100, 0, 0); "compare register a")]
    #[test_case(op::CPX_IMM, (0, 100, 0); "compare register x")]
    #[test_case(op::CPY_IMM, (0, 0, 100); "compare register y")]
    fn compare_register(opcode: u8, register_values: (u8, u8, u8)) {
        let mut memory = [opcode, 100, opcode, 99, opcode, 101]; // 0x0C
        let mut cpu = Cpu::new();
        cpu.reset();
        (cpu.register_a, cpu.register_x, cpu.register_y) = register_values;

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2);
        assert_eq!(cpu.status,  StatusFlag::Zero | StatusFlag::Carry);

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2);
        assert_eq!(cpu.status, StatusFlag::Carry);

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2);
        assert_eq!(cpu.status, StatusFlag::Negative);
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

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2);
        assert_eq!(cpu.register_a, 0b1000_1000);
        assert_eq!(cpu.status, StatusFlag::Negative);

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 4);
        assert_eq!(cpu.register_a, 0b1101_0010);
        assert_eq!(cpu.status, StatusFlag::Negative);

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 3);
        assert_eq!(cpu.register_a, 0b0111_1101);
        assert!(cpu.status.is_empty());

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2);
        assert_eq!(cpu.register_a, 0);
        assert_eq!(cpu.status,  StatusFlag::Zero);
    }

    #[test_case(0x00, 0x00, false, 0x00,  StatusFlag::Zero; "zero")] 
    #[test_case(0x50, 0x50, false, 0xa0, StatusFlag::Negative | StatusFlag::Overflow; "pos overflow")] 
    #[test_case(0xff, 0xff, false, 0xfe, StatusFlag::Carry | StatusFlag::Negative; "max no overflow")] 
    #[test_case(0x80, 0xff, false, 0x7f, StatusFlag::Carry | StatusFlag::Overflow; "neg overflow to pos")] 
    #[test_case(0x7f, 0x01, false, 0x80, StatusFlag::Negative | StatusFlag::Overflow; "classic overflow")] 
    #[test_case(0xff, 0x01, false, 0x00, StatusFlag::Carry |  StatusFlag::Zero; "wrap to zero")] 
    #[test_case(0x01, 0x01, true, 0x03, StatusFlag::empty(); "carry in")] 
    #[test_case(0x00, 0x00, true, 0x01, StatusFlag::empty(); "carry in only")] 
    #[test_case(0x40, 0x40, false, 0x80, StatusFlag::Negative | StatusFlag::Overflow; "neg result")] #[test_case(0x7f, 0x00, true, 0x80, StatusFlag::Negative | StatusFlag::Overflow; "bin_adc_carry_into_bit7")]
    #[test_case(0x80, 0x80, true, 0x01, StatusFlag::Carry | StatusFlag::Overflow; "bin_adc_bit7_plus_bit7_plus_carry")]
    #[test_case(0xff, 0x00, true, 0x00, StatusFlag::Carry |  StatusFlag::Zero; "bin_adc_zero_with_carry_out")]
    #[test_case(0x80, 0x01, false, 0x81, StatusFlag::Negative; "bin_adc_negative_no_overflow")]
    #[test_case(0x80, 0x80, false, 0x00, StatusFlag::Carry |  StatusFlag::Zero | StatusFlag::Overflow; "bin_adc_negative_plus_negative_no_overflow")]
    fn adc_binary_mode(a: u8, m: u8, c: bool, expected_a: u8, expected_status: StatusFlag)
    {
        let mut memory = [op::ADC_IMM, m];
        let mut cpu = Cpu::new();
        cpu.register_a = a;
        cpu.set_status(StatusFlag::Decimal, false);
        cpu.set_status(StatusFlag::Carry, c);

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2);
        assert_eq!(cpu.register_a, expected_a);
        assert_eq!(cpu.status, expected_status);
    }

    #[test_case(0x05, 0x03, true, 0x02, StatusFlag::Carry; "normal")]
    #[test_case(0x00, 0x01, true, 0xff, StatusFlag::Negative; "borrow")]
    #[test_case(0x80, 0x01, true, 0x7f, StatusFlag::Carry | StatusFlag::Overflow; "neg_minus_pos_overflow")]
    #[test_case(0x7f, 0xff, true, 0x80, StatusFlag::Negative | StatusFlag::Overflow; "pos_minus_neg_overflow")]
    #[test_case(0x00, 0x00, false, 0xff, StatusFlag::Negative; "extra_borrow_cin0")]
    #[test_case(0x03, 0x03, true, 0x00, StatusFlag::Carry |  StatusFlag::Zero ; "exact_zero")]
    #[test_case(0xff, 0x00, true, 0xff, StatusFlag::Carry | StatusFlag::Negative; "no_borrow_needed")]
    #[test_case(0x00, 0x00, true, 0x00, StatusFlag::Carry |  StatusFlag::Zero; "bin_sbc_zero_minus_zero_cin1")]
    #[test_case(0x00, 0x01, true, 0xff, StatusFlag::Negative; "bin_sbc_zero_minus_one")]
    #[test_case(0xff, 0x01, true, 0xfe, StatusFlag::Carry | StatusFlag::Negative; "bin_sbc_max_minus_one")]
    #[test_case(0x80, 0xff, true, 0x81, StatusFlag::Negative; "bin_sbc_min_minus_one_overflow")]
    #[test_case(0x80, 0x00, false, 0x7f, StatusFlag::Carry | StatusFlag::Overflow; "bin_sbc_carry_in_changes_overflow")]
    fn sbc_binary_mode(a: u8, m: u8, c: bool, expected_a: u8, expected_status: StatusFlag)
    {
        let mut memory = [op::SBC_IMM, m];
        let mut cpu = Cpu::new();
        cpu.register_a = a;
        cpu.set_status(StatusFlag::Decimal, false);
        cpu.set_status(StatusFlag::Carry, c);
        
        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2);
        assert_eq!(cpu.register_a, expected_a);
        assert_eq!(cpu.status, expected_status);
    }
    
    #[test_case(0x00, 0x09, false, 0x09, StatusFlag::empty(); "dec_adc_simple")]
    #[test_case(0x09, 0x01, false, 0x10, StatusFlag::empty(); "dec_adc_low_nibble_carry")]
    #[test_case(0x99, 0x01, false, 0x00, StatusFlag::Carry | StatusFlag::Negative; "dec_adc_full_carry_out")]
    #[test_case(0x99, 0x01, false, 0x00, StatusFlag::Carry | StatusFlag::Negative; "dec_adc_z_flag_bug")]
    #[test_case(0x69, 0x30, false, 0x99, StatusFlag::Negative | StatusFlag::Overflow; "dec_adc_nv_bug_99")]
    #[test_case(0x09, 0x00, true, 0x10, StatusFlag::empty(); "dec_adc_carry_in_dec")]
    #[test_case(0x58, 0x46, false, 0x04, StatusFlag::Carry | StatusFlag::Negative | StatusFlag::Overflow; "dec_adc_both_nibbles_carry")]
    #[test_case(0x99, 0x99, true, 0x99, StatusFlag::Carry | StatusFlag::Overflow; "dec_adc_max_bcd")]
    #[test_case(0x0f, 0x00, false, 0x15, StatusFlag::empty(); "dec_adc_invalid_bcd_high_nibble")]
    #[test_case(0xff, 0xff, false, 0x64, StatusFlag::Carry | StatusFlag::Negative; "dec_adc_invalid_bcd_both")]
    #[test_case(0x08, 0x01, false, 0x09, StatusFlag::empty(); "dec_adc_low_nibble_no_carry")]
    #[test_case(0x09, 0x00, true, 0x10, StatusFlag::empty(); "dec_adc_low_nibble_carry_with_carry_in")]
    #[test_case(0x40, 0x10, false, 0x50, StatusFlag::empty(); "dec_adc_high_nibble_no_carry")]
    #[test_case(0x90, 0x10, false, 0x00, StatusFlag::Carry | StatusFlag::Negative; "dec_adc_high_nibble_carry")]
    fn adc_decimal_mode(a: u8, m: u8, c: bool, expected_a: u8, expected_status: StatusFlag)
    {
        let mut memory = [op::ADC_IMM, m];
        let mut cpu = Cpu::new();
        cpu.register_a = a;
        cpu.set_status(StatusFlag::Decimal, true);
        cpu.set_status(StatusFlag::Carry, c);
        
        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2);
        assert_eq!(cpu.register_a, expected_a);
        assert_eq!(cpu.status, expected_status | StatusFlag::Decimal);
    }

    #[test_case(0x09, 0x05, true, 0x04, StatusFlag::Carry; "dec_sbc_simple")]
    #[test_case(0x10, 0x01, true, 0x09, StatusFlag::Carry; "dec_sbc_low_nibble_borrow")]
    #[test_case(0x00, 0x01, true, 0x99, StatusFlag::Negative; "dec_sbc_full_borrow")]
    #[test_case(0x00, 0x01, true, 0x99, StatusFlag::Negative; "dec_sbc_flags_match_binary")]
    #[test_case(0x50, 0x25, true, 0x25, StatusFlag::Carry; "dec_sbc_carry_in_as_no_borrow")]
    #[test_case(0x05, 0x05, true, 0x00, StatusFlag::Carry |  StatusFlag::Zero; "dec_sbc_zero_result")]
    #[test_case(0x50, 0x25, false, 0x24, StatusFlag::Carry; "dec_sbc_extra_borrow_cin0")]
    #[test_case(0xff, 0x00, true, 0xff, StatusFlag::Carry | StatusFlag::Negative; "dec_sbc_invalid_bcd")]
    #[test_case(0x00, 0x00, true, 0x00, StatusFlag::Carry |  StatusFlag::Zero; "dec_sbc_00_minus_00")]
    #[test_case(0x00, 0x01, true, 0x99, StatusFlag::Negative; "dec_sbc_00_minus_01")]
    #[test_case(0x10, 0x01, true, 0x09, StatusFlag::Carry; "dec_sbc_10_minus_01")]
    #[test_case(0x10, 0x10, true, 0x00, StatusFlag::Carry |  StatusFlag::Zero; "dec_sbc_10_minus_10")]
    #[test_case(0x99, 0x01, true, 0x98, StatusFlag::Carry | StatusFlag::Negative; "dec_sbc_99_minus_01")]
    #[test_case(0x00, 0x99, true, 0x01, StatusFlag::empty(); "dec_sbc_00_minus_99")]
    #[test_case(0x50, 0x51, true, 0x99, StatusFlag::Negative; "dec_sbc_50_minus_51")]
    #[test_case(0x99, 0x99, true, 0x00, StatusFlag::Carry |  StatusFlag::Zero; "dec_sbc_99_minus_99")]
    #[test_case(0x80, 0x01, true, 0x79, StatusFlag::Carry | StatusFlag::Overflow; "dec_sbc_negative_overflow")]
    fn sbc_decimal_mode(a: u8, m: u8, c: bool, expected_a: u8, expected_status: StatusFlag)
    {
        let mut memory = [op::SBC_IMM, m];
        let mut cpu = Cpu::new();
        cpu.register_a = a;
        cpu.set_status(StatusFlag::Decimal, true);
        cpu.set_status(StatusFlag::Carry, c);

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2);
        assert_eq!(cpu.register_a, expected_a);
        assert_eq!(cpu.status, expected_status | StatusFlag::Decimal);
    }

}

#[cfg(test)]
mod jump_tests {
    use crate::{bus::MEMORY_SIZE, cpu::{Cpu, StatusFlag}};
    use crate::instructions::op_codes as op;
    use test_case::test_case;

    #[test]
    fn jump_indirect() {
        let mut memory = [
            op::JMP_IND, 0x05, 0x00,
            op::NOP, op::NOP,
            0xAB, 0xCD,
            op::NOP, op::NOP,
            op::NOP, op::NOP];
        let mut cpu = Cpu::new();

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 5);
        assert_eq!(cpu.register_a, 0);
        assert_eq!(cpu.program_counter, 0xCDAB);
        assert!(cpu.status.is_empty());
    }

    #[test]
    fn jump_absolute() {
        let mut memory = [
            op::JMP_ABS, 0x05, 0x00,
            op::NOP, op::NOP,
            0xAB, 0xCD,
            op::NOP, op::NOP,
            op::NOP, op::NOP];
        let mut cpu = Cpu::new();

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 3);
        assert_eq!(cpu.register_a, 0);
        assert_eq!(cpu.program_counter, 0x0005);
        assert!(cpu.status.is_empty());
    }
    
    #[test]
    fn jump_indirect_absolute() {
        let mut memory = [
            op::JMP_ABS, 0x05, 0x00,
            op::NOP, op::NOP,
            op::JMP_IND, 0x0C, 0x00,
            op::NOP, op::NOP,
            op::NOP, op::NOP,
            0xAB, 0xEF];
        let mut cpu = Cpu::new();

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 3);
        assert_eq!(cpu.program_counter, 0x0005);
        assert_eq!(cpu.run_step(&mut memory).unwrap(), 5);
        assert_eq!(cpu.program_counter, 0xEFAB);
    }

    #[test_case(0x025A, 0x02, 0x5C; "Second page pc")]
    #[test_case(0x0005, 0x00, 0x07; "Zero page pc")]
    fn jsr_absolute(pc: u16, pc_high: u8, pc_low: u8) {
        let mut memory = [0; 0x0300];
        memory[pc as usize] = op::JSR;
        memory[(pc + 1) as usize] = 0xAB;
        memory[(pc + 2) as usize] = 0xCD;

        let mut cpu = Cpu::new();
        cpu.reset();
        cpu.program_counter = pc;
        assert_eq!(cpu.stack_pointer, 0xFD);        

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 6, "Op cycles");
        assert_eq!(cpu.stack_pointer, 0xFB, "Stack pointer"); 
        assert_eq!(cpu.program_counter, 0xCDAB, "Program counter");
        assert_eq!(memory[0x01FD], pc_high, "Memory 0xFD"); // pc high = 0
        assert_eq!(memory[0x01FC], pc_low, "Memory 0xFC"); // pc low = 2
    }

    
    #[test_case(0x02, 0x5C, 0x025D; "Second page stack")]
    #[test_case(0x00, 0x07, 0x0008; "Zero page stack")]
    fn rts(stack_high: u8, stack_low: u8, expected_pc: u16) {
        let mut memory = [0; 0x0300];
        memory[0x02AB] = op::RTS;
        memory[0x02AC] = 0xAB;
        memory[0x02AD] = 0xCD;

        memory[0x01AB] = stack_high;
        memory[0x01AA] = stack_low;

        let mut cpu = Cpu::new();
        cpu.reset();
        cpu.program_counter = 0x02AB;
        cpu.stack_pointer = 0xA9;       

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 6, "Op cycles");
        assert_eq!(cpu.stack_pointer, 0xAB, "Stack pointer"); 
        assert_eq!(cpu.program_counter, expected_pc, "Program counter");
    }

    #[test]
    fn brk() {
        let mut memory = [0; MEMORY_SIZE];
        memory[0x8000] = op::BRK;
        memory[0xFFFE] = 0xAB;
        memory[0xFFFF] = 0xCD;

        let mut cpu = Cpu::new();
        cpu.reset();
        cpu.program_counter = 0x8000;
        cpu.status = StatusFlag::Overflow |  StatusFlag::Zero;
        assert_eq!(cpu.stack_pointer, 0xFD, "Stack pointer before");

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 7, "Operation cycles");
        assert_eq!(cpu.program_counter, 0xCDAB, "PC after BRK");
        assert_eq!(cpu.stack_pointer, 0xFA, "Stack pointer after BRK");
        assert_eq!(cpu.status, StatusFlag::Overflow |  StatusFlag::Zero | StatusFlag::InterruptDisable, "Current status");
        assert_eq!(StatusFlag::from_bits_retain(memory[0x01FB]), StatusFlag::Overflow |  StatusFlag::Zero | StatusFlag::Break | StatusFlag::Unused, "Status on stack");
        assert_eq!(memory[0x01FC], 0x02, "pc low on stack");
        assert_eq!(memory[0x01FD], 0x80, "pc high on stack");
    }

    
    #[test]
    fn rti() {
        let mut memory = [0; MEMORY_SIZE];
        memory[0x5000] = op::RTI;
        memory[0xFFFE] = 0xAB;
        memory[0xFFFF] = 0xCD;
        memory[0x01FB] = (StatusFlag::Overflow |  StatusFlag::Zero | StatusFlag::Break | StatusFlag::Unused).bits();
        memory[0x01FC] = 0x02;
        memory[0x01FD] = 0x80;

        let mut cpu = Cpu::new();
        cpu.reset();
        cpu.program_counter = 0x5000;
        cpu.status = StatusFlag::Negative;
        cpu.stack_pointer = 0xFA;

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 6, "Operation cycles");
        assert_eq!(cpu.program_counter, 0x8002, "PC after RTI");
        assert_eq!(cpu.stack_pointer, 0xFD, "Stack pointer after RTI");
        assert_eq!(cpu.status, StatusFlag::Overflow |  StatusFlag::Zero | StatusFlag::Unused, "Current status");
    }
}

#[cfg(test)]
mod stack_tests {
    use crate::cpu::{Cpu, StatusFlag};
    use crate::instructions::op_codes as op;

    #[test]
    fn push_accumulator_stack() {
        let mut memory = [0u8; 0x0200];
        let mut cpu = Cpu::new();
        cpu.reset();
        
        assert_eq!(cpu.stack_pointer, 0xFD);
        memory[0x00] = op::PHA;
        memory[0x01] = op::PHA;

        cpu.register_a = 0xAB;
        assert_eq!(cpu.run_step(&mut memory).unwrap(), 3);
        assert_eq!(cpu.stack_pointer, 0xFC);
        assert_eq!(memory[0x01FD], 0xAB);
        assert_eq!(cpu.register_a, 0xAB);
        assert!(cpu.status.is_empty());

        cpu.register_a = 0xCD;
        assert_eq!(cpu.run_step(&mut memory).unwrap(), 3);
        assert_eq!(cpu.stack_pointer, 0xFB);
        assert_eq!(memory[0x01FC], 0xCD);
        assert_eq!(memory[0x01FD], 0xAB);
        assert_eq!(cpu.register_a, 0xCD);
        assert!(cpu.status.is_empty());
    }
    
    #[test]
    fn push_status_stack() {
        let mut memory = [0u8; 0x0200];
        let mut cpu = Cpu::new();
        cpu.reset();
        
        assert_eq!(cpu.stack_pointer, 0xFD);
        memory[0x00] = op::PHP;
        memory[0x01] = op::PHP;

        cpu.status = StatusFlag::Carry | StatusFlag::Overflow;
        assert_eq!(cpu.run_step(&mut memory).unwrap(), 3);
        assert_eq!(cpu.stack_pointer, 0xFC);
        assert_eq!(StatusFlag::from_bits_retain(memory[0x01FD]), StatusFlag::Carry | StatusFlag::Overflow | StatusFlag::Break | StatusFlag::Unused);
        assert_eq!(cpu.status, StatusFlag::Carry | StatusFlag::Overflow);

        cpu.status = StatusFlag::empty();
        assert_eq!(cpu.run_step(&mut memory).unwrap(), 3);
        assert_eq!(cpu.stack_pointer, 0xFB);
        assert_eq!(StatusFlag::from_bits_retain(memory[0x01FD]), StatusFlag::Carry | StatusFlag::Overflow | StatusFlag::Break | StatusFlag::Unused);
        assert_eq!(StatusFlag::from_bits_retain(memory[0x01FC]), StatusFlag::Break | StatusFlag::Unused);
        assert!(cpu.status.is_empty());
    }
    
    #[test]
    fn pull_accumulator_stack() {
        let mut memory = [0u8; 0x0200];
        let mut cpu = Cpu::new();
        cpu.reset();
        cpu.register_a = 0x11;
        memory[0x0000] = op::PLA;
        memory[0x0001] = op::PLA;
        memory[0x0002] = op::PLA;

        memory[0x01FB] = 0;
        memory[0x01FC] = 0b1000_0000;
        memory[0x01FD] = 0b0101_0101;

        assert_eq!(cpu.stack_pointer, 0xFD);
        cpu.stack_pointer = 0xFA;

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 4);
        assert_eq!(cpu.register_a, 0);
        assert_eq!(cpu.stack_pointer, 0xFB);
        assert_eq!(cpu.status,  StatusFlag::Zero);

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 4);
        assert_eq!(cpu.register_a, 0b1000_0000);
        assert_eq!(cpu.stack_pointer, 0xFC);
        assert_eq!(cpu.status, StatusFlag::Negative);

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 4);
        assert_eq!(cpu.register_a, 0b0101_0101);
        assert_eq!(cpu.stack_pointer, 0xFD);
        assert!(cpu.status.is_empty());
    }

    
    #[test]
    fn pull_status_stack() {
        let mut memory = [0u8; 0x0200];
        let mut cpu = Cpu::new();
        cpu.reset();
        cpu.status = StatusFlag::empty();
        memory[0x0000] = op::PLP;
        memory[0x0001] = op::PLP;
        memory[0x0002] = op::PLP;

        memory[0x01FB] = (StatusFlag::Negative | StatusFlag::Break).bits();
        memory[0x01FC] = (StatusFlag::Break).bits();
        memory[0x01FD] = (StatusFlag::Carry | StatusFlag::Decimal | StatusFlag::Break | StatusFlag::Unused).bits();

        assert_eq!(cpu.stack_pointer, 0xFD);
        cpu.stack_pointer = 0xFA;

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 4);
        assert_eq!(cpu.stack_pointer, 0xFB);
        assert_eq!(cpu.status, StatusFlag::Negative | StatusFlag::Unused);

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 4);
        assert_eq!(cpu.stack_pointer, 0xFC);
        assert_eq!(cpu.status, StatusFlag::Unused);

        assert_eq!(cpu.run_step(&mut memory).unwrap(), 4);
        assert_eq!(cpu.stack_pointer, 0xFD);
        assert_eq!(cpu.status, StatusFlag::Carry | StatusFlag::Decimal | StatusFlag::Unused);
    }
}

#[cfg(test)]
mod branch_tests {
    use crate::cpu::{Cpu, StatusFlag};
    use crate::instructions::op_codes::*;
    use test_case::test_case;

    #[test_case(BCS, StatusFlag::Carry, true; "Branch if carry set")]
    #[test_case(BCC, StatusFlag::Carry, false; "Branch if carry clear")]
    #[test_case(BEQ, StatusFlag::Zero, true; "Branch if equal")]
    #[test_case(BNE, StatusFlag::Zero, false; "Branch if not equal")]
    #[test_case(BMI, StatusFlag::Negative, true; "Branch if minus")]
    #[test_case(BPL, StatusFlag::Negative, false; "Branch if positive")]
    #[test_case(BVS, StatusFlag::Overflow, true; "Branch if overflow set")]
    #[test_case(BVC, StatusFlag::Overflow, false; "Branch if overflow clear")]
    fn take_branch(op_code: u8, flag_condition: StatusFlag, value_condition: bool) {
        let mut memory = [
            op_code, 0x05, // +5
            NOP, NOP, NOP, NOP, NOP,
            op_code, 0xE5, // -27
            op_code, 0xE5]; // -27
        let mut cpu = Cpu::new();
        cpu.reset();
        cpu.set_status(flag_condition, value_condition);

        // Branch taken, no page jump
        assert_eq!(cpu.run_step(&mut memory).unwrap(), 3, "Op cycles, branch taken, no jump");
        assert_eq!(cpu.program_counter, 0x07, "Program counter, branch taken, no jump");

        // Branch not taken
        cpu.set_status(flag_condition, !value_condition);
        assert_eq!(cpu.run_step(&mut memory).unwrap(), 2, "Op cycles, branch not taken");
        assert_eq!(cpu.program_counter, 0x09, "Program counter, branch not taken");

        // Branch taken with page jump
        cpu.set_status(flag_condition, value_condition);
        assert_eq!(cpu.run_step(&mut memory).unwrap(), 4, "Op cycles, branch taken, page jump");
        assert_eq!(cpu.program_counter, 0xFFF0, "Program counter, branch taken, page jump");
    }
}