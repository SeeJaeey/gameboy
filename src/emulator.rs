use crate::cartridge::{Cartridge};
use crate::decode::{
    CbInstruction, Cond, Instruction, R8, R16, R16Mem, R16Stk, decode, decode_cb_prefix,
};
use crate::memory::Memory;
use crate::register::{Register16, Registers};

pub struct Emulator {
    pub registers: Registers,
    pub memory: Memory,

    pub ime: bool,
    pub set_ime: bool,
    pub halt: bool,
    pub stop: bool,
}

impl Emulator {
    pub fn new() -> Self {
        Emulator {
            registers: Registers::new(),
            memory: Memory::new(),
            ime: true,
            set_ime: false,
            halt: false,
            stop: false,
        }
    }

    pub fn new_with_cartridge(cartridge: Cartridge) -> Self {
        Emulator {
            registers: Registers::new(),
            memory: Memory::new_with_cartridge(cartridge),
            ime: true,
            set_ime: false,
            halt: false,
            stop: false,
        }
    }

    pub fn step(&mut self) -> u8 {
        let ime_set_before = self.set_ime;
        
        let cycles = self.handle_interrupts();
        if cycles.is_some() {
            let cycles = cycles.unwrap();
            self.memory.tick(cycles);
            return cycles;
        }
        
        if self.halt {
            self.memory.tick(4);
            return 4;
        }
        
        let opcode = self.memory.read(self.registers.pc.get());
        let instruction = decode(opcode);
        let cycles = self.execute(instruction);

        if ime_set_before && self.set_ime {
            self.set_ime = false;
            self.ime = true;
        }

        self.memory.tick(cycles);    
        cycles
    }

    // Execute an instruction and return the number of cycles it takes
    fn execute(&mut self, instruction: Instruction) -> u8 {
        match instruction {
            Instruction::Nop => {
                self.inc_pc(1);
                4
            }

            Instruction::LdR16Imm16(r16) => {
                let imm16 = self.fetch_imm16();

                self.get_r16_mut(r16).set(imm16);

                self.inc_pc(3);
                12
            }
            Instruction::LdR16memA(r16mem) => {
                let a = self.registers.af.hi();
                let r16mem_val = self.get_r16mem_val(r16mem);

                self.memory.write(r16mem_val, a);

                self.inc_pc(1);
                8
            }
            Instruction::LdAR16mem(r16mem) => {
                let r16mem_val = self.get_r16mem_val(r16mem);
                let a_new = self.memory.read(r16mem_val);

                self.registers.af.set_hi(a_new);

                self.inc_pc(1);
                8
            }
            Instruction::LdImm16Sp => {
                let imm16 = self.fetch_imm16();
                let sp_hi = self.registers.sp.hi();
                let sp_lo = self.registers.sp.lo();

                self.memory.write(imm16, sp_lo);
                self.memory.write(imm16.wrapping_add(1), sp_hi);

                self.inc_pc(3);
                20
            }

            Instruction::IncR16(r16) => {
                let r16 = self.get_r16_mut(r16);
                r16.inc();

                self.inc_pc(1);
                8
            }
            Instruction::DecR16(r16) => {
                let r16 = self.get_r16_mut(r16);
                r16.dec();

                self.inc_pc(1);
                8
            }
            Instruction::AddHlR16(r16) => {
                let r16 = self.get_r16(r16).get();
                let hl = self.registers.hl.get();

                let result = hl.wrapping_add(r16);
                let half_carry = ((hl & 0x0FFF) + (r16 & 0x0FFF)) > 0x0FFF;
                let carry = (hl as u32 + r16 as u32) > 0xFFFF;

                self.registers.hl.set(result);
                self.registers.af.set_subtract(false);
                self.registers.af.set_half_carry(half_carry);
                self.registers.af.set_carry(carry);

                self.inc_pc(1);
                8
            }

            Instruction::IncR8(r8) => {
                let old = self.get_r8(&r8);
                let new = old.wrapping_add(1);
                self.set_r8(&r8, new);

                self.registers.af.set_zero(new == 0);
                self.registers.af.set_subtract(false);
                self.registers.af.set_half_carry((old & 0x0F) == 0x0F);

                self.inc_pc(1);
                if r8 == R8::HLIndirect { 12 } else { 4 }
            }
            Instruction::DecR8(r8) => {
                let old = self.get_r8(&r8);
                let new = old.wrapping_sub(1);
                self.set_r8(&r8, new);

                self.registers.af.set_zero(new == 0);
                self.registers.af.set_subtract(true);
                self.registers.af.set_half_carry((old & 0x0F) == 0);

                self.inc_pc(1);
                if r8 == R8::HLIndirect { 12 } else { 4 }
            }

            Instruction::LdR8Imm8(r8) => {
                let imm8 = self.fetch_imm8();
                self.set_r8(&r8, imm8);

                self.inc_pc(2);
                if r8 == R8::HLIndirect { 12 } else { 8 }
            }

            Instruction::Rlca => {
                let a = self.registers.af.hi();
                let bit7 = a >> 7 & 1;
                let rotation_res = (a << 1) | bit7;

                self.registers.af.set_hi(rotation_res);
                self.registers.af.set_flags(false, false, false, bit7 != 0);

                self.inc_pc(1);
                4
            }
            Instruction::Rrca => {
                let a = self.registers.af.hi();
                let bit0 = a & 1;
                let rotation_res = (a >> 1) | (bit0 << 7);

                self.registers.af.set_hi(rotation_res);
                self.registers.af.set_flags(false, false, false, bit0 != 0);

                self.inc_pc(1);
                4
            }
            Instruction::Rla => {
                let a = self.registers.af.hi();
                let bit7 = a >> 7 & 1;
                let rotation_res = (a << 1) | self.registers.af.carry() as u8;

                self.registers.af.set_hi(rotation_res);
                self.registers.af.set_flags(false, false, false, bit7 != 0);

                self.inc_pc(1);
                4
            }
            Instruction::Rra => {
                let a = self.registers.af.hi();
                let bit0 = a & 1;
                let rotation_res = (a >> 1) | ((self.registers.af.carry() as u8) << 7);

                self.registers.af.set_hi(rotation_res);
                self.registers.af.set_flags(false, false, false, bit0 != 0);

                self.inc_pc(1);
                4
            }
            Instruction::Daa => {
                let mut a = self.registers.af.hi();
                let subtract = self.registers.af.subtract();
                let half_carry = self.registers.af.half_carry();
                let mut carry = self.registers.af.carry();

                if !subtract {
                    if carry || a > 0x99 {
                        a = a.wrapping_add(0x60);
                        carry = true;
                    }
                    if half_carry || (a & 0x0F) > 0x09 {
                        a = a.wrapping_add(0x06);
                    }
                } else {
                    if carry {
                        a = a.wrapping_sub(0x60);
                    }
                    if half_carry {
                        a = a.wrapping_sub(0x06);
                    }
                }

                self.registers.af.set_hi(a);
                self.registers.af.set_zero(a == 0);
                self.registers.af.set_half_carry(false);
                self.registers.af.set_carry(carry);

                self.inc_pc(1);
                4
            }
            Instruction::Cpl => {
                let a = self.registers.af.hi();
                self.registers.af.set_hi(!a);

                self.registers.af.set_subtract(true);
                self.registers.af.set_half_carry(true);

                self.inc_pc(1);
                4
            }
            Instruction::Scf => {
                self.registers.af.set_carry(true);
                self.registers.af.set_subtract(false);
                self.registers.af.set_half_carry(false);

                self.inc_pc(1);
                4
            }
            Instruction::Ccf => {
                let c = self.registers.af.carry();
                self.registers.af.set_carry(!c);
                self.registers.af.set_subtract(false);
                self.registers.af.set_half_carry(false);

                self.inc_pc(1);
                4
            }

            Instruction::JrImm8 => {
                let imm8 = self.fetch_imm8() as i8;

                self.inc_pc(2);
                let new_pc = self.registers.pc.get().wrapping_add_signed(imm8 as i16);
                self.registers.pc.set(new_pc);
                12
            }
            Instruction::JrCondImm8(cond) => {
                let imm8 = self.fetch_imm8() as i8;
                self.inc_pc(2);

                if self.check_condition(cond) {
                    let new_pc = self.registers.pc.get().wrapping_add_signed(imm8 as i16);
                    self.registers.pc.set(new_pc);
                    12
                } else {
                    8
                }
            }

            Instruction::Stop => {
                // TODO: also stops internal counter?
                4
            }

            Instruction::LdR8R8(lr8, rr8) => {
                let value = self.get_r8(&rr8);
                self.set_r8(&lr8, value);

                self.inc_pc(1);
                if lr8 == R8::HLIndirect || rr8 == R8::HLIndirect {
                    8
                } else {
                    4
                }
            }
            Instruction::Halt => {
                self.halt = true;

                self.inc_pc(1);
                4
            }

            Instruction::AddAR8(r8) => {
                let a = self.registers.af.hi();
                let value = self.get_r8(&r8);

                let result = a.wrapping_add(value);
                let half_carry = ((a & 0x0F) + (value & 0x0F)) > 0x0F;
                let carry = a as u16 + value as u16 > 0xFF;

                self.registers.af.set_hi(result);
                self.registers
                    .af
                    .set_flags(result == 0, false, half_carry, carry);

                self.inc_pc(1);
                if r8 == R8::HLIndirect { 8 } else { 4 }
            }
            Instruction::AdcAR8(r8) => {
                let a = self.registers.af.hi();
                let value = self.get_r8(&r8);
                let carry = self.registers.af.carry() as u8;

                let result = a.wrapping_add(value).wrapping_add(carry);
                let half_carry = (a & 0x0F) + (value & 0x0F) + carry > 0x0F;
                let carry = a as u16 + value as u16 + carry as u16 > 0xFF;

                self.registers.af.set_hi(result);
                self.registers
                    .af
                    .set_flags(result == 0, false, half_carry, carry);

                self.inc_pc(1);
                if r8 == R8::HLIndirect { 8 } else { 4 }
            }
            Instruction::SubAR8(r8) => {
                let a = self.registers.af.hi();
                let value = self.get_r8(&r8);

                let result = a.wrapping_sub(value);
                let half_carry = (a & 0x0F) < (value & 0x0F);

                self.registers.af.set_hi(result);
                self.registers
                    .af
                    .set_flags(result == 0, true, half_carry, a < value);

                self.inc_pc(1);
                if r8 == R8::HLIndirect { 8 } else { 4 }
            }
            Instruction::SbcAR8(r8) => {
                let a = self.registers.af.hi();
                let value = self.get_r8(&r8);
                let carry_u8 = self.registers.af.carry() as u8;

                let result = a.wrapping_sub(value).wrapping_sub(carry_u8);
                let half_carry = (a & 0x0F) < (value & 0x0F) + carry_u8;
                let carry = (a as u16) < (value as u16) + carry_u8 as u16;

                self.registers.af.set_hi(result);
                self.registers
                    .af
                    .set_flags(result == 0, true, half_carry, carry);

                self.inc_pc(1);
                if r8 == R8::HLIndirect { 8 } else { 4 }
            }
            Instruction::AndAR8(r8) => {
                let a = self.registers.af.hi();
                let value = self.get_r8(&r8);

                let result = a & value;

                self.registers.af.set_hi(result);
                self.registers.af.set_flags(result == 0, false, true, false);

                self.inc_pc(1);
                if r8 == R8::HLIndirect { 8 } else { 4 }
            }
            Instruction::XorAR8(r8) => {
                let a = self.registers.af.hi();
                let value = self.get_r8(&r8);

                let result = a ^ value;

                self.registers.af.set_hi(result);
                self.registers
                    .af
                    .set_flags(result == 0, false, false, false);

                self.inc_pc(1);
                if r8 == R8::HLIndirect { 8 } else { 4 }
            }
            Instruction::OrAR8(r8) => {
                let a = self.registers.af.hi();
                let value = self.get_r8(&r8);

                let result = a | value;

                self.registers.af.set_hi(result);
                self.registers
                    .af
                    .set_flags(result == 0, false, false, false);

                self.inc_pc(1);
                if r8 == R8::HLIndirect { 8 } else { 4 }
            }
            Instruction::CpAR8(r8) => {
                let a = self.registers.af.hi();
                let value = self.get_r8(&r8);

                let result = a.wrapping_sub(value);
                let half_carry = (a & 0x0F) < (value & 0x0F);

                self.registers
                    .af
                    .set_flags(result == 0, true, half_carry, a < value);

                self.inc_pc(1);
                if r8 == R8::HLIndirect { 8 } else { 4 }
            }

            Instruction::AddAImm8 => {
                let a = self.registers.af.hi();
                let value = self.fetch_imm8();

                let result = a.wrapping_add(value);

                let half_carry = (a & 0x0F) + (value & 0x0F) > 0x0F;
                let carry = (a as u16 + value as u16) > 0xFF;

                self.registers.af.set_hi(result);
                self.registers
                    .af
                    .set_flags(result == 0, false, half_carry, carry);

                self.inc_pc(2);
                8
            }
            Instruction::AdcAImm8 => {
                let a = self.registers.af.hi();
                let value = self.fetch_imm8();
                let carry = self.registers.af.carry() as u8;

                let result = a.wrapping_add(value).wrapping_add(carry);
                let half_carry = (a & 0x0F) + (value & 0x0F) + carry > 0x0F;
                let carry = (a as u16 + value as u16 + carry as u16) > 0xFF;

                self.registers.af.set_hi(result);
                self.registers
                    .af
                    .set_flags(result == 0, false, half_carry, carry);

                self.inc_pc(2);
                8
            }
            Instruction::SubAImm8 => {
                let a = self.registers.af.hi();
                let value = self.fetch_imm8();

                let result = a.wrapping_sub(value);
                let half_carry = (a & 0x0F) < (value & 0x0F);

                self.registers.af.set_hi(result);
                self.registers
                    .af
                    .set_flags(result == 0, true, half_carry, a < value);

                self.inc_pc(2);
                8
            }
            Instruction::SbcAImm8 => {
                let a = self.registers.af.hi();
                let value = self.fetch_imm8();
                let carry = self.registers.af.carry() as u8;

                let result = a.wrapping_sub(value).wrapping_sub(carry);
                let half_carry = (a & 0x0F) < (value & 0x0F) + carry;
                let carry = (a as u16) < (value as u16 + carry as u16);

                self.registers.af.set_hi(result);
                self.registers
                    .af
                    .set_flags(result == 0, true, half_carry, carry);

                self.inc_pc(2);
                8
            }
            Instruction::AndAImm8 => {
                let a = self.registers.af.hi();
                let value = self.fetch_imm8();

                let result = a & value;

                self.registers.af.set_hi(result);
                self.registers.af.set_flags(result == 0, false, true, false);

                self.inc_pc(2);
                8
            }
            Instruction::XorAImm8 => {
                let a = self.registers.af.hi();
                let value = self.fetch_imm8();

                let result = a ^ value;

                self.registers.af.set_hi(result);
                self.registers
                    .af
                    .set_flags(result == 0, false, false, false);

                self.inc_pc(2);
                8
            }
            Instruction::OrAImm8 => {
                let a = self.registers.af.hi();
                let value = self.fetch_imm8();

                let result = a | value;

                self.registers.af.set_hi(result);
                self.registers
                    .af
                    .set_flags(result == 0, false, false, false);

                self.inc_pc(2);
                8
            }
            Instruction::CpAImm8 => {
                let a = self.registers.af.hi();
                let value = self.fetch_imm8();

                let result = a.wrapping_sub(value);
                let half_carry = (a & 0x0F) < (value & 0x0F);

                self.registers
                    .af
                    .set_flags(result == 0, true, half_carry, a < value);

                self.inc_pc(2);
                8
            }

            Instruction::RetCond(cond) => {
                if self.check_condition(cond) {
                    let new_pc = self.pop_u16_from_mem();
                    self.registers.pc.set(new_pc);
                    20
                } else {
                    self.inc_pc(1);
                    8
                }
            }
            Instruction::Ret => {
                let new_pc = self.pop_u16_from_mem();
                self.registers.pc.set(new_pc);
                16
            }
            Instruction::Reti => {
                self.ime = true;

                let new_pc = self.pop_u16_from_mem();
                self.registers.pc.set(new_pc);
                16
            }
            Instruction::JpCondImm16(cond) => {
                let address = self.fetch_imm16();

                if self.check_condition(cond) {
                    self.registers.pc.set(address);
                    16
                } else {
                    self.inc_pc(3);
                    12
                }
            }
            Instruction::JpImm16 => {
                let address = self.fetch_imm16();
                self.registers.pc.set(address);
                16
            }
            Instruction::JpHl => {
                self.registers.pc.set(self.registers.hl.get());
                4
            }
            Instruction::CallCondImm16(cond) => {
                let address = self.fetch_imm16();

                if self.check_condition(cond) {
                    self.inc_pc(3);
                    self.push_u16_to_mem(self.registers.pc.get());
                    self.registers.pc.set(address);

                    24
                } else {
                    self.inc_pc(3);
                    12
                }
            }
            Instruction::CallImm16 => {
                let address = self.fetch_imm16();

                self.inc_pc(3);
                self.push_u16_to_mem(self.registers.pc.get());
                self.registers.pc.set(address);

                24
            }
            Instruction::RstTgt3(target) => {
                self.inc_pc(1);
                self.push_u16_to_mem(self.registers.pc.get());
                self.registers.pc.set(target as u16);

                16
            }

            Instruction::PopR16stk(r16stk) => {
                let value = self.pop_u16_from_mem();

                match r16stk {
                    R16Stk::Bc => self.registers.bc.set(value),
                    R16Stk::De => self.registers.de.set(value),
                    R16Stk::Hl => self.registers.hl.set(value),
                    R16Stk::Af => self.registers.af.set(value & 0xFFF0),
                }

                self.inc_pc(1);
                12
            }
            Instruction::PushR16stk(r16stk) => {
                let value = match r16stk {
                    R16Stk::Bc => self.registers.bc.get(),
                    R16Stk::De => self.registers.de.get(),
                    R16Stk::Hl => self.registers.hl.get(),
                    R16Stk::Af => self.registers.af.get(),
                };

                self.push_u16_to_mem(value);

                self.inc_pc(1);
                16
            }

            Instruction::Prefix => {
                let opcode = self.fetch_imm8();
                let instruction = decode_cb_prefix(opcode);
                self.inc_pc(2);
                self.execute_cb(instruction)
            }

            Instruction::LdhCA => {
                let c = self.registers.bc.lo();
                let addr = 0xFF00 + c as u16;
                let a = self.registers.af.hi();

                self.memory.write(addr, a);

                self.inc_pc(1);
                8
            }
            Instruction::LdhImm8A => {
                let offset = self.fetch_imm8();
                let addr = 0xFF00 + offset as u16;
                let a = self.registers.af.hi();

                self.memory.write(addr, a);

                self.inc_pc(2);
                12
            }
            Instruction::LdImm16A => {
                let addr = self.fetch_imm16();
                let a = self.registers.af.hi();

                self.memory.write(addr, a);

                self.inc_pc(3);
                16
            }
            Instruction::LdhAC => {
                let c = self.registers.bc.lo();
                let addr = 0xFF00 + c as u16;

                let value = self.memory.read(addr);
                self.registers.af.set_hi(value);

                self.inc_pc(1);
                8
            }
            Instruction::LdhAImm8 => {
                let offset = self.fetch_imm8();
                let addr = 0xFF00 + offset as u16;

                let value = self.memory.read(addr);
                self.registers.af.set_hi(value);

                self.inc_pc(2);
                12
            }
            Instruction::LdAImm16 => {
                let addr = self.fetch_imm16();
                let value = self.memory.read(addr);

                self.registers.af.set_hi(value);

                self.inc_pc(3);
                16
            }

            Instruction::AddSpImm8 => {
                let sp = self.registers.sp.get();
                let imm8_u = self.fetch_imm8();
                let imm8 = imm8_u as i8;

                let result = sp.wrapping_add_signed(imm8 as i16);

                let half_carry = ((sp & 0x000F) + (imm8_u as u16 & 0x000F)) > 0x000F;
                let carry = ((sp & 0x00FF) + (imm8_u as u16 & 0x00FF)) > 0x00FF;

                self.registers.sp.set(result);
                self.registers.af.set_flags(false, false, half_carry, carry);

                self.inc_pc(2);
                16
            }
            Instruction::LdHlSpPlusImm8 => {
                let sp = self.registers.sp.get();
                let imm8_u = self.fetch_imm8();
                let imm8 = imm8_u as i8;

                let result = sp.wrapping_add_signed(imm8 as i16);

                let half_carry = ((sp & 0x000F) + (imm8_u as u16 & 0x000F)) > 0x000F;
                let carry = ((sp & 0x00FF) + (imm8_u as u16 & 0x00FF)) > 0x00FF;

                self.registers.hl.set(result);
                self.registers.af.set_flags(false, false, half_carry, carry);

                self.inc_pc(2);
                12
            }
            Instruction::LdSpHl => {
                let hl = self.registers.hl.get();
                self.registers.sp.set(hl);

                self.inc_pc(1);
                8
            }

            Instruction::Di => {
                self.ime = false;

                self.inc_pc(1);
                4
            }
            Instruction::Ei => {
                if !self.ime {
                    self.set_ime = true;
                }

                self.inc_pc(1);
                4
            }
        }
    }

    fn execute_cb(&mut self, instruction: CbInstruction) -> u8 {
        match instruction {
            CbInstruction::RlcR8(r8) => {
                let value = self.get_r8(&r8);
                let bit7 = (value >> 7) & 1;
                let result = (value << 1) | bit7;

                self.set_r8(&r8, result);
                self.registers
                    .af
                    .set_flags(result == 0, false, false, bit7 != 0);

                if r8 == R8::HLIndirect { 16 } else { 8 }
            }
            CbInstruction::RrcR8(r8) => {
                let value = self.get_r8(&r8);
                let bit0 = value & 1;
                let result = (value >> 1) | (bit0 << 7);

                self.set_r8(&r8, result);
                self.registers
                    .af
                    .set_flags(result == 0, false, false, bit0 != 0);

                if r8 == R8::HLIndirect { 16 } else { 8 }
            }
            CbInstruction::RlR8(r8) => {
                let value = self.get_r8(&r8);
                let bit7 = (value >> 7) & 1;
                let old_carry = self.registers.af.carry() as u8;
                let result = (value << 1) | old_carry;

                self.set_r8(&r8, result);
                self.registers
                    .af
                    .set_flags(result == 0, false, false, bit7 != 0);

                if r8 == R8::HLIndirect { 16 } else { 8 }
            }
            CbInstruction::RrR8(r8) => {
                let value = self.get_r8(&r8);
                let bit0 = value & 1;
                let old_carry = self.registers.af.carry() as u8;
                let result = (value >> 1) | (old_carry << 7);

                self.set_r8(&r8, result);
                self.registers
                    .af
                    .set_flags(result == 0, false, false, bit0 != 0);

                if r8 == R8::HLIndirect { 16 } else { 8 }
            }
            CbInstruction::SlaR8(r8) => {
                let value = self.get_r8(&r8);
                let bit7 = (value >> 7) & 1;
                let result = value << 1;

                self.set_r8(&r8, result);
                self.registers
                    .af
                    .set_flags(result == 0, false, false, bit7 != 0);

                if r8 == R8::HLIndirect { 16 } else { 8 }
            }
            CbInstruction::SraR8(r8) => {
                let value = self.get_r8(&r8);
                let bit0 = value & 1;
                let sign_bit = value & 0x80;
                let result = (value >> 1) | sign_bit;

                self.set_r8(&r8, result);
                self.registers
                    .af
                    .set_flags(result == 0, false, false, bit0 != 0);

                if r8 == R8::HLIndirect { 16 } else { 8 }
            }
            CbInstruction::SwapR8(r8) => {
                let value = self.get_r8(&r8);
                let result = (value << 4) | (value >> 4);

                self.set_r8(&r8, result);
                self.registers
                    .af
                    .set_flags(result == 0, false, false, false);

                if r8 == R8::HLIndirect { 16 } else { 8 }
            }
            CbInstruction::SrlR8(r8) => {
                let value = self.get_r8(&r8);
                let bit0 = value & 1;
                let result = value >> 1;

                self.set_r8(&r8, result);
                self.registers
                    .af
                    .set_flags(result == 0, false, false, bit0 != 0);

                if r8 == R8::HLIndirect { 16 } else { 8 }
            }

            CbInstruction::BitB3R8(bit, r8) => {
                let value = self.get_r8(&r8);
                let is_set = (value >> bit) & 1 != 0;

                self.registers.af.set_zero(!is_set);
                self.registers.af.set_subtract(false);
                self.registers.af.set_half_carry(true);

                if r8 == R8::HLIndirect { 12 } else { 8 }
            }
            CbInstruction::ResB3R8(bit, r8) => {
                let value = self.get_r8(&r8);
                let result = value & !(1 << bit);

                self.set_r8(&r8, result);

                if r8 == R8::HLIndirect { 16 } else { 8 }
            }
            CbInstruction::SetB3R8(bit, r8) => {
                let value = self.get_r8(&r8);
                let result = value | (1 << bit);

                self.set_r8(&r8, result);

                if r8 == R8::HLIndirect { 16 } else { 8 }
            }
        }
    }

    fn get_r8(&self, r8: &R8) -> u8 {
        match r8 {
            R8::B => self.registers.bc.hi(),
            R8::C => self.registers.bc.lo(),
            R8::D => self.registers.de.hi(),
            R8::E => self.registers.de.lo(),
            R8::H => self.registers.hl.hi(),
            R8::L => self.registers.hl.lo(),
            R8::A => self.registers.af.hi(),
            R8::HLIndirect => {
                let addr = self.registers.hl.get();
                self.memory.read(addr)
            }
        }
    }

    fn set_r8(&mut self, r8: &R8, value: u8) {
        match r8 {
            R8::B => self.registers.bc.set_hi(value),
            R8::C => self.registers.bc.set_lo(value),
            R8::D => self.registers.de.set_hi(value),
            R8::E => self.registers.de.set_lo(value),
            R8::H => self.registers.hl.set_hi(value),
            R8::L => self.registers.hl.set_lo(value),
            R8::A => self.registers.af.set_hi(value),
            R8::HLIndirect => {
                let addr = self.registers.hl.get();
                self.memory.write(addr, value);
            }
        }
    }

    fn get_r16_mut(&mut self, r16: R16) -> &mut Register16 {
        match r16 {
            R16::Bc => &mut self.registers.bc,
            R16::De => &mut self.registers.de,
            R16::Hl => &mut self.registers.hl,
            R16::Sp => &mut self.registers.sp,
        }
    }

    fn get_r16(&self, r16: R16) -> &Register16 {
        match r16 {
            R16::Bc => &self.registers.bc,
            R16::De => &self.registers.de,
            R16::Hl => &self.registers.hl,
            R16::Sp => &self.registers.sp,
        }
    }

    fn get_r16mem_val(&mut self, r16mem: R16Mem) -> u16 {
        match r16mem {
            R16Mem::Bc => self.registers.bc.get(),
            R16Mem::De => self.registers.de.get(),
            R16Mem::HlInc => {
                let tmp = self.registers.hl.get();
                self.registers.hl.inc();
                tmp
            }
            R16Mem::HlDec => {
                let tmp = self.registers.hl.get();
                self.registers.hl.dec();
                tmp
            }
        }
    }

    fn inc_pc(&mut self, value: u16) {
        self.registers
            .pc
            .set(self.registers.pc.get().wrapping_add(value));
    }

    fn fetch_imm8(&self) -> u8 {
        self.memory.read(self.registers.pc.get().wrapping_add(1))
    }

    fn fetch_imm16(&self) -> u16 {
        let lo = self.memory.read(self.registers.pc.get().wrapping_add(1)) as u16;
        let hi = self.memory.read(self.registers.pc.get().wrapping_add(2)) as u16;
        (hi << 8) | lo
    }

    fn check_condition(&self, cond: Cond) -> bool {
        match cond {
            Cond::Nz => !self.registers.af.zero(),
            Cond::Z => self.registers.af.zero(),
            Cond::Nc => !self.registers.af.carry(),
            Cond::C => self.registers.af.carry(),
        }
    }

    fn push_u16_to_mem(&mut self, value: u16) {
        self.registers.sp.dec();
        self.memory.write(self.registers.sp.get(), (value >> 8) as u8);

        self.registers.sp.dec();
        self.memory.write(self.registers.sp.get(), value as u8);
    }

    fn pop_u16_from_mem(&mut self) -> u16 {
        let lo = self.memory.read(self.registers.sp.get()) as u16;
        self.registers.sp.inc();

        let hi = self.memory.read(self.registers.sp.get()) as u16;
        self.registers.sp.inc();

        (hi << 8) | lo
    }

    fn handle_interrupts(&mut self) -> Option<u8> {
        let interrupt = self.memory.get_interrupt();
        if interrupt.is_none() {
            return None;
        }
        let interrupt = interrupt.unwrap();

        self.halt = false;

        if !self.ime {
            return None;
        }

        self.ime = false;
        self.push_u16_to_mem(self.registers.pc.get());
        self.registers.pc.set(interrupt.address());
        self.memory.unset_if(interrupt);
    
        Some(20)
    }
}
