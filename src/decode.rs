// Reference: https://gbdev.io/pandocs/CPU_Instruction_Set.html

// TODO: from opcode für alle enums

pub enum R8 {
    B, C, D, E, H, L,
    HLIndirect, // memory[HL]
    A,
}

impl R8 {
    pub fn from_bits(bits: u8) -> Self {
        match bits & 0b111 {
            0 => R8::B,
            1 => R8::C,
            2 => R8::D,
            3 => R8::E,
            4 => R8::H,
            5 => R8::L,
            6 => R8::HLIndirect,
            7 => R8::A,
            _ => unreachable!(),
        }
    }

    pub fn dst_from_opcode(byte: u8) -> Self {
        Self::from_bits(byte >> 3)
    }

    pub fn src_from_opcode(byte: u8) -> Self {
        Self::from_bits(byte)
    }
}

pub enum R16 {
    Bc, De, Hl, Sp,
}

impl R16 {
    pub fn from_bits(bits: u8) -> Self {
        match bits & 0b11 {
            0 => R16::Bc,
            1 => R16::De,
            2 => R16::Hl,
            3 => R16::Sp,
            _ => unreachable!(),
        }
    }

    pub fn from_opcode(byte: u8) -> Self {
        Self::from_bits(byte >> 4)
    }
}

pub enum R16Stk {
    Bc, De, Hl, Af,
}

impl R16Stk {
    pub fn from_bits(bits: u8) -> Self {
        match bits & 0b11 {
            0 => R16Stk::Bc,
            1 => R16Stk::De,
            2 => R16Stk::Hl,
            3 => R16Stk::Af,
            _ => unreachable!(),
        }
    }

    pub fn from_opcode(byte: u8) -> Self {
        Self::from_bits(byte >> 4)
    }
}

pub enum R16Mem {
    Bc, De, HlInc, HlDec,
}

impl R16Mem {
    pub fn from_bits(bits: u8) -> Self {
        match bits & 0b11 {
            0 => R16Mem::Bc,
            1 => R16Mem::De,
            2 => R16Mem::HlInc,
            3 => R16Mem::HlDec,
            _ => unreachable!(),
        }
    }

    pub fn from_opcode(byte: u8) -> Self {
        Self::from_bits(byte >> 4)
    }
}

pub enum Cond {
    Nz, Z, Nc, C,
}

impl Cond {
    pub fn from_bits(bits: u8) -> Self {
        match bits & 0b11 {
            0 => Cond::Nz,
            1 => Cond::Z,
            2 => Cond::Nc,
            3 => Cond::C,
            _ => unreachable!(),
        }
    }

    pub fn from_opcode(byte: u8) -> Self {
        Self::from_bits(byte >> 3)
    }
}

pub enum Instruction {
    /////////////
    // BLOCK 0 // --> 63 instructions
    /////////////

    // 1
    Nop,
    // 12
    LdR16Imm16(R16), // Group of 4
    LdR16memA(R16Mem), // Group of 4,
    LdAR16mem(R16Mem), // Group of 4,
    LdImm16Sp,
    // 12
    IncR16(R16), // Group of 4
    DecR16(R16), // Group of 4
    AddHlR16(R16), // Group of 4
    // 16
    IncR8(R8), // Group of 8
    DecR8(R8), // Group of 8
    // 8
    LdR8Imm8(R8), // Group of 8
    // 8
    Rlca,
    Rrca,
    Rla,
    Rra,
    Daa,
    Cpl,
    Scf,
    Ccf,
    // 5
    JrImm8,
    JrCondImm8(Cond), // Group of 4
    // 1
    Stop,

    /////////////
    // BLOCK 1 // --> 64 instructions
    /////////////

    // 63
    LdR8R8(R8, R8), // Group of 63
    // 1
    Halt,

    /////////////
    // BLOCK 2 // --> 64 instructions
    /////////////

    // 64
    AddAR8(R8), // Group of 8
    AdcAR8(R8), // Group of 8
    SubAR8(R8), // Group of 8
    SbcAR8(R8), // Group of 8
    AndAR8(R8), // Group of 8
    XorAR8(R8), // Group of 8
    OrAR8(R8), // Group of 8
    CpAR8(R8), // Group of 8

    /////////////
    // BLOCK 3 // --> 53 instructions
    /////////////

    // 8
    AddAImm8,
    AdcAImm8,
    SubAImm8,
    SbcAImm8,
    AndAImm8,
    XorAImm8,
    OrAImm8,
    CpAImm8,

    // 25
    RetCond(Cond), // Group of 4
    Ret,
    Reti,
    JpCondImm16(Cond), // Group of 4
    JpImm16,
    JpHl,
    CallCondImm16(Cond), // Group of 4
    CallImm16,
    RstTgt3(u8), // Group of 8, u8 is target

    // 8
    PopR16stk(R16Stk), // Group of 4
    PushR16stk(R16Stk), // Group of 4

    // 1
    Prefix,

    // 6
    LdhCA,
    LdhImm8A,
    LdImm16A,
    LdhAC,
    LdhAImm8,
    LdAImm16,

    // 3
    AddSpImm8,
    LdHlSpPlusImm8,
    LdSpHl,

    // 2
    Di,
    Ei,

    ////////////////
    // $CB prefix // --> 256 instructions
    ////////////////

    // 64
    RlcR8(R8), // Group of 4
    RrcR8(R8), // Group of 4
    RlR8(R8), // Group of 4
    RrR8(R8), // Group of 4
    SlaR8(R8), // Group of 4
    SraR8(R8), // Group of 4
    SwapR8(R8), // Group of 4
    SrlR8(R8), // Group of 4

    // 192
    BitB3R8(u8, R8), // Group of 64, u8 is for bit index
    ResB3R8(u8, R8), // Group of 64, u8 is for bit index
    SetB3R8(u8, R8), // Group of 64, u8 is for bit index
}

pub fn decode(opcode: u8) -> Instruction {
    match opcode {
        0x00 => Instruction::Nop,

        0x01 | 0x11 | 0x21 | 0x31 => Instruction::LdR16Imm16(R16::from_opcode(opcode)),
        0x02 | 0x12 | 0x22 | 0x32 => Instruction::LdR16memA(R16Mem::from_opcode(opcode)),
        0x0A | 0x1A | 0x2A | 0x3A => Instruction::LdAR16mem(R16Mem::from_opcode(opcode)),
        0x08 => Instruction::LdImm16Sp,

        0x03 | 0x13 | 0x23 | 0x33 => Instruction::IncR16(R16::from_opcode(opcode)),
        0x0B | 0x1B | 0x2B | 0x3B => Instruction::DecR16(R16::from_opcode(opcode)),
        0x09 | 0x19 | 0x29 | 0x39 => Instruction::AddHlR16(R16::from_opcode(opcode)),

        0x04 | 0x14 | 0x24 | 0x34 | 0x0C | 0x1C | 0x2C | 0x3C => Instruction::IncR8(R8::dst_from_opcode(opcode)),
        0x05 | 0x15 | 0x25 | 0x35 | 0x0D | 0x1D | 0x2D | 0x3D => Instruction::DecR8(R8::dst_from_opcode(opcode)),

        0x06 | 0x16 | 0x26 | 0x36 | 0x0E | 0x1E | 0x2E | 0x3E => Instruction::LdR8Imm8(R8::dst_from_opcode(opcode)),

        0x07 => Instruction::Rlca,
        0x0F => Instruction::Rrca,
        0x17 => Instruction::Rla,
        0x1F => Instruction::Rra,
        0x27 => Instruction::Daa,
        0x2F => Instruction::Cpl,
        0x37 => Instruction::Scf,
        0x3F => Instruction::Ccf,

        0x18 => Instruction::JrImm8,
        0x20 | 0x30 | 0x28 | 0x38 => Instruction::JrCondImm8(Cond::from_opcode(opcode)),

        0x10 => Instruction::Stop,

        0x40..=0x75 | 0x77..=0x7F => Instruction::LdR8R8(R8::dst_from_opcode(opcode), R8::src_from_opcode(opcode)),

        0x76 => Instruction::Halt,

        0x80..=0x87 => Instruction::AddAR8(R8::src_from_opcode(opcode)),
        0x88..=0x8F => Instruction::AdcAR8(R8::src_from_opcode(opcode)),
        0x90..=0x97 => Instruction::SubAR8(R8::src_from_opcode(opcode)),
        0x98..=0x9F => Instruction::SbcAR8(R8::src_from_opcode(opcode)),
        0xA0..=0xA7 => Instruction::AndAR8(R8::src_from_opcode(opcode)),
        0xA8..=0xAF => Instruction::XorAR8(R8::src_from_opcode(opcode)),
        0xB0..=0xB7 => Instruction::OrAR8(R8::src_from_opcode(opcode)),
        0xB8..=0xBF => Instruction::CpAR8(R8::src_from_opcode(opcode)),

        0xC6 => Instruction::AddAImm8,
        0xCE => Instruction::AdcAImm8,
        0xD6 => Instruction::SubAImm8,
        0xDE => Instruction::SbcAImm8,
        0xE6 => Instruction::AndAImm8,
        0xEE => Instruction::XorAImm8,
        0xF6 => Instruction::OrAImm8,
        0xFE => Instruction::CpAImm8,

        0xC0 | 0xD0 | 0xC8 | 0xD8 => Instruction::RetCond(Cond::from_opcode(opcode)),
        0xC9 => Instruction::Ret,
        0xD9 => Instruction::Reti,
        0xC2 | 0xD2 | 0xCA | 0xDA => Instruction::JpCondImm16(Cond::from_opcode(opcode)),
        0xC3 => Instruction::JpImm16,
        0xE9 => Instruction::JpHl,
        0xC4 | 0xD4 | 0xCC | 0xDC => Instruction::CallCondImm16(Cond::from_opcode(opcode)),
        0xCD => Instruction::CallImm16,
        0xC7 | 0xD7 | 0xE7 | 0xF7 | 0xCF | 0xDF | 0xEF | 0xFF => Instruction::RstTgt3((opcode & 0b0011_1000) << 3), // TODO: divide by 8 or not?

        0xC1 | 0xD1 | 0xE1 | 0xF1 => Instruction::PopR16stk(R16Stk::from_opcode(opcode)),
        0xC5 | 0xD5 | 0xE5 | 0xF5 => Instruction::PushR16stk(R16Stk::from_opcode(opcode)),

        0xCB => Instruction::Prefix,

        0xE0 => Instruction::LdhImm8A,
        0xF0 => Instruction::LdhAImm8,
        0xE2 => Instruction::LdhCA,
        0xF2 => Instruction::LdhAC,
        0xEA => Instruction::LdImm16A,
        0xFA => Instruction::LdAImm16,

        0xE8 => Instruction::AddSpImm8,
        0xF8 => Instruction::LdHlSpPlusImm8,
        0xF9 => Instruction::LdSpHl,

        0xF3 => Instruction::Di,
        0xFB => Instruction::Ei,

        0xD3 | 0xDB | 0xDD | 0xE3 | 0xE4 | 0xEB | 0xEC | 0xED | 0xF4 | 0xFC | 0xFD => panic!("Encountered invalid opcode which hard-locked the CPU: {}", opcode),
    }
}

pub fn decode_cb_prefix(opcode: u8) -> Instruction {
    let bit = (opcode & 0b0011_1000) << 3; // TODO: divide by 8 or not?

    match opcode {
        0x00..=0x07 => Instruction::RlcR8(R8::src_from_opcode(opcode)),
        0x08..=0x0F => Instruction::RrcR8(R8::src_from_opcode(opcode)),
        0x10..=0x17 => Instruction::RlR8(R8::src_from_opcode(opcode)),
        0x18..=0x1F => Instruction::RrR8(R8::src_from_opcode(opcode)),
        0x20..=0x27 => Instruction::SlaR8(R8::src_from_opcode(opcode)),
        0x28..=0x2F => Instruction::SraR8(R8::src_from_opcode(opcode)),
        0x30..=0x37 => Instruction::SwapR8(R8::src_from_opcode(opcode)),
        0x38..=0x3F => Instruction::SrlR8(R8::src_from_opcode(opcode)),

        0x40..=0x7F => Instruction::BitB3R8(bit, R8::src_from_opcode(opcode)),
        0x80..=0xBF => Instruction::ResB3R8(bit, R8::src_from_opcode(opcode)),
        0xC0..=0xFF => Instruction::ResB3R8(bit, R8::src_from_opcode(opcode)),
    }
}