use std::fmt::Display;

use crate::address::AddressMode;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mnemonic {
    ADC, AND, ASL, BCC, BCS, BEQ, BIT, BMI, BNE, BPL, BRK, BVC, BVS, CLC,
    CLD, CLI, CLV, CMP, CPX, CPY, DEC, DEX, DEY, EOR, INC, INX, INY, JMP,
    JSR, LDA, LDX, LDY, LSR, NOP, ORA, PHA, PHP, PLA, PLP, ROL, ROR, RTI,
    RTS, SBC, SEC, SED, SEI, STA, STX, STY, TAX, TAY, TSX, TXA, TXS, TYA
}

impl Display for Mnemonic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Instruction {
    pub address_mode: AddressMode,
    pub mnemonic: Mnemonic,
    pub cycles: u8,
    pub can_cross_page: bool
}

macro_rules! instructions {
    ($( $name:ident : $code:expr => ($mnemenic:ident, $addr:ident, $cycles:expr, $cross:expr) ),* $(,)?) => {
        pub mod op_codes {
            $(
                pub const $name: u8 = $code;
            )*
        }

        pub(crate) static INSTRUCTION_SET: [Option<Instruction>; 0x100] = {
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
    INC_ABX: 0xFE => (INC, AbsoluteX,   7, false),

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
    
    ADC_IMM: 0x69 => (ADC, Immediate,   2, false),
    ADC_ZER: 0x65 => (ADC, ZeroPage,    3, false),
    ADC_ZEX: 0x75 => (ADC, ZeroPageX,   4, false),
    ADC_ABS: 0x6D => (ADC, Absolute,    4, false),
    ADC_ABX: 0x7D => (ADC, AbsoluteX,   4, true),
    ADC_ABY: 0x79 => (ADC, AbsoluteY,   4, true),
    ADC_INX: 0x61 => (ADC, IndirectX,   6, false),
    ADC_INY: 0x71 => (ADC, IndirectY,   5, true),

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
    
    BCC    : 0x90 => (BCC, Relative,    2, true),
    BCS    : 0xB0 => (BCS, Relative,    2, true),
    BEQ    : 0xF0 => (BEQ, Relative,    2, true),
    BMI    : 0x30 => (BMI, Relative,    2, true),
    BNE    : 0xD0 => (BNE, Relative,    2, true),
    BPL    : 0x10 => (BPL, Relative,    2, true),
    BVC    : 0x50 => (BVC, Relative,    2, true),
    BVS    : 0x70 => (BVS, Relative,    2, true),

    BIT_ZER: 0x24 => (BIT, ZeroPage,    3, false), 
    BIT_ABS: 0x2C => (BIT, Absolute,    4, false),

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

    JSR    : 0x20 => (JSR, Absolute,    6, false),

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

    PHA    : 0x48 => (PHA, Implied,     3, false),
    PHP    : 0x08 => (PHP, Implied,     3, false),
    PLA    : 0x68 => (PLA, Implied,     4, false),
    PLP    : 0x28 => (PLP, Implied,     4, false),

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

    RTI    : 0x40 => (RTI, Implied,     6, false),
    RTS    : 0x60 => (RTS, Implied,     6, false),

    SBC_IMM: 0xE9 => (SBC, Immediate,	2, false), 
    SBC_ZER: 0xE5 => (SBC, ZeroPage,	3, false), 
    SBC_ZEX: 0xF5 => (SBC, ZeroPageX,	4, false), 
    SBC_ABS: 0xED => (SBC, Absolute,	4, false), 
    SBC_ABX: 0xFD => (SBC, AbsoluteX,	4, true),
    SBC_ABY: 0xF9 => (SBC, AbsoluteY,	4, true),
    SBC_INX: 0xE1 => (SBC, IndirectX,	6, false),
    SBC_INY: 0xF1 => (SBC, IndirectY,	5, true),
    
    SEC    : 0x38 => (SEC, Implied,     2, false),
    SED    : 0xF8 => (SED, Implied,     2, false),
    SEI    : 0x78 => (SEI, Implied,     2, false),

    STA_ZER: 0x85 => (STA, ZeroPage,    3, false),
    STA_ZEX: 0x95 => (STA, ZeroPageX,   4, false),
    STA_ABS: 0x8D => (STA, Absolute,    4, false),
    STA_ABX: 0x9D => (STA, AbsoluteX,   5, false),
    STA_ABY: 0x99 => (STA, AbsoluteY,   5, false),
    STA_INX: 0x81 => (STA, IndirectX,   6, false),
    STA_INY: 0x91 => (STA, IndirectY,   6, false),

    STX_ZER: 0x86 => (STX, ZeroPage,    3, false),
    STX_ZEY: 0x96 => (STX, ZeroPageY,   4, false),
    STX_ABS: 0x8E => (STX, Absolute,    4, false),

    STY_ZER: 0x84 => (STY, ZeroPage,    3, false), 
    STY_ZEX: 0x94 => (STY, ZeroPageX,   4, false), 
    STY_ABS: 0x8C => (STY, Absolute,    4, false),

    TAX    : 0xAA => (TAX, Implied,     2, false),
    TAY    : 0xA8 => (TAY, Implied,     2, false),
    TSX    : 0xBA => (TSX, Implied,     2, false),
    TXA    : 0x8A => (TXA, Implied,     2, false),
    TXS    : 0x9A => (TXS, Implied,     2, false),
    TYA    : 0x98 => (TYA, Implied,     2, false),

}
