use crate::register::{Registers, Register16};
use crate::decode::{Instruction, R8, R16, R16Mem, R16Stk, Cond};
use crate::memory::Memory;

pub struct Emulator {
    pub registers: Registers,
    pub memory: Memory,
}

impl Emulator {
    // Execute an instruction and return the number of cycles it takes
    pub fn execute(&mut self, instruction: Instruction) -> u8 {
        match instruction {
            // TODO: implement execution block for each instruction
            Instruction::Nop => {
                self.inc_pc(1);
                4
            }

            Instruction::LdR16Imm16(r16) => {
                let imm16 = self.fetch_imm16();

                self.get_r16_mut(r16).set(imm16);

                self.inc_pc(3);
                8
            },
            Instruction::LdR16memA(r16mem) => {
                let a = self.registers.af.hi();
                let r16mem_val = self.get_r16mem_val(r16mem);

                self.memory.set(r16mem_val, a);

                self.inc_pc(1);
                8
            },
            Instruction::LdAR16mem(r16mem) => {
                let r16mem_val = self.get_r16mem_val(r16mem);
                let a_new = self.memory.get(r16mem_val);

                self.registers.af.set_hi(a_new);

                self.inc_pc(1);
                8
            },
            Instruction::LdImm16Sp => {
                let imm16 = self.fetch_imm16();
                let sp_hi = self.registers.sp.hi();
                let sp_lo = self.registers.sp.lo();

                self.memory.set(imm16, sp_lo);
                self.memory.set(imm16.wrapping_add(1), sp_hi);

                self.inc_pc(3);
                20
            },

            Instruction::IncR16(r16) => {
                let r16 = self.get_r16_mut(r16);
                r16.inc();

                self.inc_pc(1);
                8
            },
            Instruction::DecR16(r16) => {
                let r16 = self.get_r16_mut(r16);
                r16.dec();

                self.inc_pc(1);
                8
            },
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
            },

            Instruction::IncR8(r8) => {
                let old = match r8 {
                    R8::B => self.registers.bc.hi(),
                    R8::C => self.registers.bc.lo(),
                    R8::D => self.registers.de.hi(),
                    R8::E => self.registers.de.lo(),
                    R8::H => self.registers.hl.hi(),
                    R8::L => self.registers.hl.lo(),
                    R8::A => self.registers.af.hi(),
                    R8::HLIndirect => {
                        let addr = self.registers.hl.get();
                        self.memory.get(addr)
                    }
                };

                let new = old.wrapping_add(1);
                self.set_r8(&r8, new);

                self.registers.af.set_zero(new == 0);
                self.registers.af.set_subtract(false);
                self.registers.af.set_half_carry((old & 0x0F) == 0x0F);

                self.inc_pc(1);
                if r8 == R8::HLIndirect { 12 } else { 4 }
            },
            Instruction::DecR8(r8) => {
                let old = match r8 {
                    R8::B => self.registers.bc.hi(),
                    R8::C => self.registers.bc.lo(),
                    R8::D => self.registers.de.hi(),
                    R8::E => self.registers.de.lo(),
                    R8::H => self.registers.hl.hi(),
                    R8::L => self.registers.hl.lo(),
                    R8::A => self.registers.af.hi(),
                    R8::HLIndirect => {
                        let addr = self.registers.hl.get();
                        self.memory.get(addr)
                    }
                };

                let new = old.wrapping_sub(1);
                self.set_r8(&r8, new);

                self.registers.af.set_zero(new == 0);
                self.registers.af.set_subtract(true);
                self.registers.af.set_half_carry((old & 0x0F) == 0);

                self.inc_pc(1);
                if r8 == R8::HLIndirect { 12 } else { 4 }
            },

            Instruction::LdR8Imm8(r8) => {
                let imm8 = self.fetch_imm8();

                match r8 {
                    R8::B => self.registers.bc.set_hi(imm8),
                    R8::C => self.registers.bc.set_lo(imm8),
                    R8::D => self.registers.de.set_hi(imm8),
                    R8::E => self.registers.de.set_lo(imm8),
                    R8::H => self.registers.hl.set_hi(imm8),
                    R8::L => self.registers.hl.set_lo(imm8),
                    R8::HLIndirect => {
                        let addr = self.registers.hl.get();
                        self.memory.set(addr, imm8);
                    }
                    R8::A => self.registers.af.set_hi(imm8),
                }

                self.inc_pc(2);
                if r8 == R8::HLIndirect { 12 } else { 8 }
            },

            Instruction::Rlca => {
                let a = self.registers.af.hi();
                let bit7 = a >> 7 & 1;
                let rotation_res = (a << 1) | bit7;

                self.registers.af.set_hi(rotation_res);
                self.registers.af.set_flags(false, false, false, bit7 != 0);

                self.inc_pc(1);
                4
            },
            Instruction::Rrca => {
                let a = self.registers.af.hi();
                let bit0 = a & 1;
                let rotation_res = (a >> 1) | (bit0 << 7);

                self.registers.af.set_hi(rotation_res);
                self.registers.af.set_flags(false, false, false, bit0 != 0);

                self.inc_pc(1);
                4
            },
            Instruction::Rla => {
                let a = self.registers.af.hi();
                let bit7 = a >> 7 & 1;
                let rotation_res = (a << 1) | self.registers.af.carry() as u8;

                self.registers.af.set_hi(rotation_res);
                self.registers.af.set_flags(false, false, false, bit7 != 0);

                self.inc_pc(1);
                4
            },
            Instruction::Rra => {
                let a = self.registers.af.hi();
                let bit0 = a & 1;
                let rotation_res = (a >> 1) | ((self.registers.af.carry() as u8) << 7);

                self.registers.af.set_hi(rotation_res);
                self.registers.af.set_flags(false, false, false, bit0 != 0);

                self.inc_pc(1);
                4
            },
            Instruction::Daa => {
                // TODO
                4
            },
            Instruction::Cpl => {
                let a = self.registers.af.hi();
                self.registers.af.set_hi(!a);

                self.registers.af.set_subtract(true);
                self.registers.af.set_half_carry(true);

                self.inc_pc(1);
                4
            },
            Instruction::Scf => {
                self.registers.af.set_carry(true);
                self.registers.af.set_subtract(false);
                self.registers.af.set_half_carry(false);

                self.inc_pc(1);
                4
            },
            Instruction::Ccf => {
                let c = self.registers.af.carry();
                self.registers.af.set_carry(!c);
                self.registers.af.set_subtract(false);
                self.registers.af.set_half_carry(false);

                self.inc_pc(1);
                4
            },

            Instruction::JrImm8 => {
                let imm8 = self.fetch_imm8() as i8;

                self.inc_pc(2);
                let new_pc = self.registers.pc.get().wrapping_add_signed(imm8 as i16);
                self.registers.pc.set(new_pc);
                12
            },
            Instruction::JrCondImm8(cond) => {
                let condition = match cond {
                    Cond::Nz => !self.registers.af.zero(),
                    Cond::Z => self.registers.af.zero(),
                    Cond::Nc => !self.registers.af.carry(),
                    Cond::C => self.registers.af.carry(),
                };

                let imm8 = self.fetch_imm8() as i8;
                self.inc_pc(2);

                if condition {
                    let new_pc = self.registers.pc.get().wrapping_add_signed(imm8 as i16);
                    self.registers.pc.set(new_pc);
                    12
                } else {
                    8
                }
            },

            Instruction::Stop => {
                // TODO
                4
            }



            Instruction::LdR8R8(lr8, rr8) => {
                let value = self.get_r8(&rr8);
                self.set_r8(&lr8, value);

                self.inc_pc(1);
                if lr8 == R8::HLIndirect || rr8 == R8::HLIndirect { 8 } else { 4 }
            },
            // 1
            Instruction::Halt => {
                // TODO
                4
            },



            Instruction::AddAR8(r8) => {
                let a = self.registers.af.hi();
                let value = self.get_r8(&r8);

                let result = a.wrapping_add(value);

                self.registers.af.set_hi(result);
                self.registers.af.set_zero(result == 0);
                self.registers.af.set_subtract(false);
                self.registers.af.set_half_carry(((a & 0x0F) + (value & 0x0F)) > 0x0F);
                self.registers.af.set_carry(a as u16 + value as u16 > 0xFF);

                self.inc_pc(1);
                if r8 == R8::HLIndirect { 8 } else { 4 }
            },
            Instruction::AdcAR8(r8) => {
                let a = self.registers.af.hi();
                let value = self.get_r8(&r8);
                let carry = self.registers.af.carry() as u8;

                let result = a.wrapping_add(value).wrapping_add(carry);

                self.registers.af.set_hi(result);
                self.registers.af.set_zero(result == 0);
                self.registers.af.set_subtract(false);
                self.registers.af.set_half_carry(
                    (a & 0x0F) + (value & 0x0F) + carry > 0x0F
                );
                self.registers.af.set_carry(
                    a as u16 + value as u16 + carry as u16 > 0xFF
                );

                self.inc_pc(1);
                if r8 == R8::HLIndirect { 8 } else { 4 }
            },
            Instruction::SubAR8(r8) => {
                let a = self.registers.af.hi();
                let value = self.get_r8(&r8);

                let result = a.wrapping_sub(value);

                self.registers.af.set_hi(result);
                self.registers.af.set_zero(result == 0);
                self.registers.af.set_subtract(true);
                self.registers.af.set_half_carry((a & 0x0F) < (value & 0x0F));
                self.registers.af.set_carry(a < value);

                self.inc_pc(1);
                if r8 == R8::HLIndirect { 8 } else { 4 }
            },
            Instruction::SbcAR8(r8) => {
                let a = self.registers.af.hi();
                let value = self.get_r8(&r8);
                let carry = self.registers.af.carry() as u8;

                let result = a.wrapping_sub(value).wrapping_sub(carry);

                self.registers.af.set_hi(result);
                self.registers.af.set_zero(result == 0);
                self.registers.af.set_subtract(true);
                self.registers.af.set_half_carry(
                    (a & 0x0F) < (value & 0x0F) + carry
                );
                self.registers.af.set_carry(
                    (a as u16) < (value as u16) + carry as u16
                );

                self.inc_pc(1);
                if r8 == R8::HLIndirect { 8 } else { 4 }
            },
            Instruction::AndAR8(r8) => {
                let a = self.registers.af.hi();
                let value = self.get_r8(&r8);

                let result = a & value;

                self.registers.af.set_hi(result);
                self.registers.af.set_zero(result == 0);
                self.registers.af.set_subtract(false);
                self.registers.af.set_half_carry(true);
                self.registers.af.set_carry(false);

                self.inc_pc(1);
                if r8 == R8::HLIndirect { 8 } else { 4 }
            },
            Instruction::XorAR8(r8) => {
                let a = self.registers.af.hi();
                let value = self.get_r8(&r8);

                let result = a ^ value;

                self.registers.af.set_hi(result);
                self.registers.af.set_zero(result == 0);
                self.registers.af.set_subtract(false);
                self.registers.af.set_half_carry(false);
                self.registers.af.set_carry(false);

                self.inc_pc(1);
                if r8 == R8::HLIndirect { 8 } else { 4 }
            },
            Instruction::OrAR8(r8) => {
                let a = self.registers.af.hi();
                let value = self.get_r8(&r8);

                let result = a | value;

                self.registers.af.set_hi(result);
                self.registers.af.set_zero(result == 0);
                self.registers.af.set_subtract(false);
                self.registers.af.set_half_carry(false);
                self.registers.af.set_carry(false);

                self.inc_pc(1);
                if r8 == R8::HLIndirect { 8 } else { 4 }
            },
            Instruction::CpAR8(r8) => {
                let a = self.registers.af.hi();
                let value = self.get_r8(&r8);

                let result = a.wrapping_sub(value);

                self.registers.af.set_zero(result == 0);
                self.registers.af.set_subtract(true);
                self.registers.af.set_half_carry((a & 0x0F) < (value & 0x0F));
                self.registers.af.set_carry(a < value);

                self.inc_pc(1);
                if r8 == R8::HLIndirect { 8 } else { 4 }
            },


            // TODO: start implementing block 3
            Instruction::AddAImm8 => {
                8
            },
            Instruction::AdcAImm8 => {
                8
            },
            Instruction::SubAImm8 => {
                8
            },
            Instruction::SbcAImm8 => {
                8
            },
            Instruction::AndAImm8 => {
                8
            },
            Instruction::XorAImm8 => {
                8
            },
            Instruction::OrAImm8 => {
                8
            },
            Instruction::CpAImm8 => {
                8
            },

            Instruction::RetCond(cond) => {
                20 // 20/8
            },
            Instruction::Ret => {
                16
            },
            Instruction::Reti => {
                16
            },
            Instruction::JpCondImm16(cond) => {
                16 // 16/12
            },
            Instruction::JpImm16 => {
                16
            },
            Instruction::JpHl => {
                4
            },
            Instruction::CallCondImm16(cond) => {
                24 // 24/12
            },
            Instruction::CallImm16 => {
                24
            },
            Instruction::RstTgt3(u8) => { // u8 is target
                16
            },

            Instruction::PopR16stk(r16stk) => {
                12
            },
            Instruction::PushR16stk(r16stk) => {
                16
            },

            Instruction::Prefix => {
                4
            },

            Instruction::LdhCA => {
                8
            },
            Instruction::LdhImm8A => {
                12
            },
            Instruction::LdImm16A => {
                16
            },
            Instruction::LdhAC => {
                8
            },
            Instruction::LdhAImm8 => {
                12
            },
            Instruction::LdAImm16 => {
                16
            },

            Instruction::AddSpImm8 => {
                16
            },
            Instruction::LdHlSpPlusImm8 => {
                12
            },
            Instruction::LdSpHl => {
                8
            },

            Instruction::Di => {
                4
            },
            Instruction::Ei => {
                4
            },



            Instruction::RlcR8(r8) => {
                if r8 == R8::HLIndirect { 16 } else { 8 }
            },
            Instruction::RrcR8(r8) => {
                if r8 == R8::HLIndirect { 16 } else { 8 }
            },
            Instruction::RlR8(r8) => {
                if r8 == R8::HLIndirect { 16 } else { 8 }
            },
            Instruction::RrR8(r8) => {
                if r8 == R8::HLIndirect { 16 } else { 8 }
            },
            Instruction::SlaR8(r8) => {
                if r8 == R8::HLIndirect { 16 } else { 8 }
            },
            Instruction::SraR8(r8) => {
                if r8 == R8::HLIndirect { 16 } else { 8 }
            },
            Instruction::SwapR8(r8) => {
                if r8 == R8::HLIndirect { 16 } else { 8 }
            },
            Instruction::SrlR8(r8) => {
                if r8 == R8::HLIndirect { 16 } else { 8 }
            },
            
            Instruction::BitB3R8(u8, r8) => {
                if r8 == R8::HLIndirect { 16 } else { 8 }
            },
            Instruction::ResB3R8(u8, r8) => {
                if r8 == R8::HLIndirect { 16 } else { 8 }
            },
            Instruction::SetB3R8(u8, r8) => {
                if r8 == R8::HLIndirect { 16 } else { 8 }
            },
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
                self.memory.get(addr)
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
                self.memory.set(addr, value);
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
        self.registers.pc.set(self.registers.pc.get().wrapping_add(value));
    }

    pub fn fetch_imm8(&self) -> u8 {
        self.memory.0[(self.registers.pc.get().wrapping_add(1)) as usize]
    }

    pub fn fetch_imm16(&self) -> u16 {
        let lo = self.memory.0[(self.registers.pc.get().wrapping_add(1)) as usize] as u16;
        let hi = self.memory.0[(self.registers.pc.get().wrapping_add(2)) as usize] as u16;
        (hi << 8) | lo
    }
}
