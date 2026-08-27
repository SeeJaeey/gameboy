use crate::decode::R8;
use crate::cpu::Registers;

/* fn read_r8(r8: R8, registers: &Registers, memory: &impl MemoryBus) -> u8 {
    match r8 {
        R8::A => registers.a(),
        R8::B => registers.bc.hi(),
        R8::C => registers.bc.lo(),
        R8::D => registers.de.hi(),
        R8::E => registers.de.lo(),
        R8::H => registers.hl.hi(),
        R8::L => registers.hl.lo(),
        R8::HLIndirect => memory.read(registers.hl.get()),
    }
}

fn write_r8(r8: R8, value: u8, registers: &mut Registers, memory: &mut impl MemoryBus) {
    match r8 {
        R8::A => registers.set_a(value),
        R8::B => registers.bc.set_hi(value),
        R8::C => registers.bc.set_lo(value),
        R8::D => registers.de.set_hi(value),
        R8::E => registers.de.set_lo(value),
        R8::H => registers.hl.set_hi(value),
        R8::L => registers.hl.set_lo(value),
        R8::HLIndirect => memory.write(registers.hl.get(), value),
    }
} */