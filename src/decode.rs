use crate::cpu::{Registers, Register16};

pub enum Instruction {
    // Reference: https://gbdev.io/pandocs/CPU_Instruction_Set.html
    Nop,

    LdR16Imm16(Register16), // Group of 4
    LdR16memA(Register16, i8), // Group of 4, i8 used for hl+/hl-
    LdAR16mem(Register16, i8), // Group of 4, i8 used for hl+/hl-
    LdImm16Sp(Register16),
}

pub fn decode(instruction: u16, registers: Registers) -> Instruction {
    let hi = (instruction >> 8) as u8;
    let lo = (instruction & 0x00FF) as u8; // Used if hi = 0xCB

    match hi {
        0x00 => Instruction::Nop,

        0x01 => Instruction::LdR16Imm16(registers.bc),
        0x11 => Instruction::LdR16Imm16(registers.de),
        0x21 => Instruction::LdR16Imm16(registers.hl),
        0x31 => Instruction::LdR16Imm16(registers.sp),

        0x02 => Instruction::LdR16memA(registers.bc, 0),
        0x12 => Instruction::LdR16memA(registers.de, 0),
        0x22 => Instruction::LdR16memA(registers.hl, 1),
        0x32 => Instruction::LdR16memA(registers.hl, -1),

        0x0A => Instruction::LdAR16mem(registers.bc, 0),
        0x1A => Instruction::LdAR16mem(registers.de, 0),
        0x2A => Instruction::LdAR16mem(registers.hl, 1),
        0x3A => Instruction::LdAR16mem(registers.hl, -1),

        0x08 => Instruction::LdImm16Sp(registers.sp),

        _ => Instruction::Nop
    }
}