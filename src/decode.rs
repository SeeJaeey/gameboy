// Reference: https://gbdev.io/pandocs/CPU_Instruction_Set.html
pub enum R8 {
    B, C, D, E, H, L,
    HLIndirect, // memory[HL]
    A,
}

pub enum R16 {
    Bc, De, Hl, Sp,
}

pub enum R16Stk {
    Bc, De, Hl, Af,
}

pub enum R16Mem {
    Bc, De, HlInc, HlDec,
}

pub enum Cond {
    Nz, Z, Nc, C,
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
    RstRgt3(u8), // Group of 8, u8 is target

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

    // TODO: encode instructions that hard lock the CPU as a rust panic

    ////////////////
    // $CB prefix // --> 256 instructions
    ////////////////

    // TODO
    
}

pub fn decode(opcode: u8) -> Instruction {
    let tgt3 = (opcode & 0b0011_1000) << 3;

    match opcode {
        0x00 => Instruction::Nop,

        0x01 => Instruction::LdR16Imm16(R16::Bc),
        0x11 => Instruction::LdR16Imm16(R16::De),
        0x21 => Instruction::LdR16Imm16(R16::Hl),
        0x31 => Instruction::LdR16Imm16(R16::Sp),

        0x02 => Instruction::LdR16memA(R16Mem::Bc),
        0x12 => Instruction::LdR16memA(R16Mem::De),
        0x22 => Instruction::LdR16memA(R16Mem::HlInc),
        0x32 => Instruction::LdR16memA(R16Mem::HlDec),

        0x0A => Instruction::LdAR16mem(R16Mem::Bc),
        0x1A => Instruction::LdAR16mem(R16Mem::De),
        0x2A => Instruction::LdAR16mem(R16Mem::HlInc),
        0x3A => Instruction::LdAR16mem(R16Mem::HlDec),

        0x08 => Instruction::LdImm16Sp,

        0x03 => Instruction::IncR16(R16::Bc),
        0x13 => Instruction::IncR16(R16::De),
        0x23 => Instruction::IncR16(R16::Hl),
        0x33 => Instruction::IncR16(R16::Sp),

        0x0B => Instruction::DecR16(R16::Bc),
        0x1B => Instruction::DecR16(R16::De),
        0x2B => Instruction::DecR16(R16::Hl),
        0x3B => Instruction::DecR16(R16::Sp),

        0x09 => Instruction::AddHlR16(R16::Bc),
        0x19 => Instruction::AddHlR16(R16::De),
        0x29 => Instruction::AddHlR16(R16::Hl),
        0x39 => Instruction::AddHlR16(R16::Sp),

        0x04 => Instruction::IncR8(R8::B),
        0x14 => Instruction::IncR8(R8::D),
        0x24 => Instruction::IncR8(R8::H),
        0x34 => Instruction::IncR8(R8::HLIndirect),
        0x0C => Instruction::IncR8(R8::C),
        0x1C => Instruction::IncR8(R8::E),
        0x2C => Instruction::IncR8(R8::L),
        0x3C => Instruction::IncR8(R8::A),

        0x05 => Instruction::DecR8(R8::B),
        0x15 => Instruction::DecR8(R8::D),
        0x25 => Instruction::DecR8(R8::H),
        0x35 => Instruction::DecR8(R8::HLIndirect),
        0x0D => Instruction::DecR8(R8::C),
        0x1D => Instruction::DecR8(R8::E),
        0x2D => Instruction::DecR8(R8::L),
        0x3D => Instruction::DecR8(R8::A),

        0x06 => Instruction::LdR8Imm8(R8::B),
        0x16 => Instruction::LdR8Imm8(R8::D),
        0x26 => Instruction::LdR8Imm8(R8::H),
        0x36 => Instruction::LdR8Imm8(R8::HLIndirect),
        0x0E => Instruction::LdR8Imm8(R8::C),
        0x1E => Instruction::LdR8Imm8(R8::E),
        0x2E => Instruction::LdR8Imm8(R8::L),
        0x3E => Instruction::LdR8Imm8(R8::A),

        0x07 => Instruction::Rlca,
        0x0F => Instruction::Rrca,
        0x17 => Instruction::Rla,
        0x1F => Instruction::Rra,
        0x27 => Instruction::Daa,
        0x2F => Instruction::Cpl,
        0x37 => Instruction::Scf,
        0x3F => Instruction::Ccf,

        0x18 => Instruction::JrImm8,

        0x20 => Instruction::JrCondImm8(Cond::Nz),
        0x30 => Instruction::JrCondImm8(Cond::Nc),
        0x28 => Instruction::JrCondImm8(Cond::Z),
        0x38 => Instruction::JrCondImm8(Cond::C),

        0x10 => Instruction::Stop,

        0x40 => Instruction::LdR8R8(R8::B, R8::B),
        0x41 => Instruction::LdR8R8(R8::B, R8::C),
        0x42 => Instruction::LdR8R8(R8::B, R8::D),
        0x43 => Instruction::LdR8R8(R8::B, R8::E),
        0x44 => Instruction::LdR8R8(R8::B, R8::H),
        0x45 => Instruction::LdR8R8(R8::B, R8::L),
        0x46 => Instruction::LdR8R8(R8::B, R8::HLIndirect),
        0x47 => Instruction::LdR8R8(R8::B, R8::A),
        0x48 => Instruction::LdR8R8(R8::C, R8::B),
        0x49 => Instruction::LdR8R8(R8::C, R8::C),
        0x4A => Instruction::LdR8R8(R8::C, R8::D),
        0x4B => Instruction::LdR8R8(R8::C, R8::E),
        0x4C => Instruction::LdR8R8(R8::C, R8::H),
        0x4D => Instruction::LdR8R8(R8::C, R8::L),
        0x4E => Instruction::LdR8R8(R8::C, R8::HLIndirect),
        0x4F => Instruction::LdR8R8(R8::C, R8::A),

        0x50 => Instruction::LdR8R8(R8::D, R8::B),
        0x51 => Instruction::LdR8R8(R8::D, R8::C),
        0x52 => Instruction::LdR8R8(R8::D, R8::D),
        0x53 => Instruction::LdR8R8(R8::D, R8::E),
        0x54 => Instruction::LdR8R8(R8::D, R8::H),
        0x55 => Instruction::LdR8R8(R8::D, R8::L),
        0x56 => Instruction::LdR8R8(R8::D, R8::HLIndirect),
        0x57 => Instruction::LdR8R8(R8::D, R8::A),
        0x58 => Instruction::LdR8R8(R8::E, R8::B),
        0x59 => Instruction::LdR8R8(R8::E, R8::C),
        0x5A => Instruction::LdR8R8(R8::E, R8::D),
        0x5B => Instruction::LdR8R8(R8::E, R8::E),
        0x5C => Instruction::LdR8R8(R8::E, R8::H),
        0x5D => Instruction::LdR8R8(R8::E, R8::L),
        0x5E => Instruction::LdR8R8(R8::E, R8::HLIndirect),
        0x5F => Instruction::LdR8R8(R8::E, R8::A),

        0x60 => Instruction::LdR8R8(R8::H, R8::B),
        0x61 => Instruction::LdR8R8(R8::H, R8::C),
        0x62 => Instruction::LdR8R8(R8::H, R8::D),
        0x63 => Instruction::LdR8R8(R8::H, R8::E),
        0x64 => Instruction::LdR8R8(R8::H, R8::H),
        0x65 => Instruction::LdR8R8(R8::H, R8::L),
        0x66 => Instruction::LdR8R8(R8::H, R8::HLIndirect),
        0x67 => Instruction::LdR8R8(R8::H, R8::A),
        0x68 => Instruction::LdR8R8(R8::L, R8::B),
        0x69 => Instruction::LdR8R8(R8::L, R8::C),
        0x6A => Instruction::LdR8R8(R8::L, R8::D),
        0x6B => Instruction::LdR8R8(R8::L, R8::E),
        0x6C => Instruction::LdR8R8(R8::L, R8::H),
        0x6D => Instruction::LdR8R8(R8::L, R8::L),
        0x6E => Instruction::LdR8R8(R8::L, R8::HLIndirect),
        0x6F => Instruction::LdR8R8(R8::L, R8::A),

        0x70 => Instruction::LdR8R8(R8::HLIndirect, R8::B),
        0x71 => Instruction::LdR8R8(R8::HLIndirect, R8::C),
        0x72 => Instruction::LdR8R8(R8::HLIndirect, R8::D),
        0x73 => Instruction::LdR8R8(R8::HLIndirect, R8::E),
        0x74 => Instruction::LdR8R8(R8::HLIndirect, R8::H),
        0x75 => Instruction::LdR8R8(R8::HLIndirect, R8::L),
        0x76 => Instruction::Halt,
        0x77 => Instruction::LdR8R8(R8::HLIndirect, R8::A),
        0x78 => Instruction::LdR8R8(R8::A, R8::B),
        0x79 => Instruction::LdR8R8(R8::A, R8::C),
        0x7A => Instruction::LdR8R8(R8::A, R8::D),
        0x7B => Instruction::LdR8R8(R8::A, R8::E),
        0x7C => Instruction::LdR8R8(R8::A, R8::H),
        0x7D => Instruction::LdR8R8(R8::A, R8::L),
        0x7E => Instruction::LdR8R8(R8::A, R8::HLIndirect),
        0x7F => Instruction::LdR8R8(R8::A, R8::A),



        _ => Instruction::Nop
    }
}

pub fn decode_cb_prefix(opcode: u8) -> Instruction {
    match opcode {
        _ => Instruction::Nop
    }
}