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
            Instruction::Nop => 4,

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
                self.memory.set(imm16 + 1, sp_hi);

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
            // TODO: Set flags
            Instruction::AddHlR16(r16) => {
                let r16 = self.get_r16(r16).get();
                let hl = self.registers.hl.get();

                self.registers.hl.set(hl.wrapping_add(r16));

                self.inc_pc(1);
                8
            },

            Instruction::IncR8(r8) => {
                match r8 {
                    R8::B => {
                        let old_b = self.registers.bc.hi();
                        self.registers.bc.inc_hi();
                        let new_b = self.registers.bc.hi();
            
                        self.registers.af.set_zero(new_b == 0);
                        self.registers.af.set_subtract(false);
                        self.registers.af.set_half_carry((old_b & 0x0F) == 0x0F);
                    }
                    R8::C => {
                        let old_c = self.registers.bc.lo();
                        self.registers.bc.inc_lo();
                        let new_c = self.registers.bc.lo();
            
                        self.registers.af.set_zero(new_c == 0);
                        self.registers.af.set_subtract(false);
                        self.registers.af.set_half_carry((old_c & 0x0F) == 0x0F);
                    }
                    R8::D => {
                        let old_d = self.registers.de.hi();
                        self.registers.de.inc_hi();
                        let new_d = self.registers.de.hi();
            
                        self.registers.af.set_zero(new_d == 0);
                        self.registers.af.set_subtract(false);
                        self.registers.af.set_half_carry((old_d & 0x0F) == 0x0F);
                    }
                    R8::E => {
                        let old_e = self.registers.de.lo();
                        self.registers.de.inc_lo();
                        let new_e = self.registers.de.lo();
            
                        self.registers.af.set_zero(new_e == 0);
                        self.registers.af.set_subtract(false);
                        self.registers.af.set_half_carry((old_e & 0x0F) == 0x0F);
                    }
                    R8::H => {
                        let old_h = self.registers.hl.hi();
                        self.registers.hl.inc_hi();
                        let new_h = self.registers.hl.hi();
            
                        self.registers.af.set_zero(new_h == 0);
                        self.registers.af.set_subtract(false);
                        self.registers.af.set_half_carry((old_h & 0x0F) == 0x0F);
                    }
                    R8::L => {
                        let old_l = self.registers.hl.lo();
                        self.registers.hl.inc_lo();
                        let new_l = self.registers.hl.lo();
            
                        self.registers.af.set_zero(new_l == 0);
                        self.registers.af.set_subtract(false);
                        self.registers.af.set_half_carry((old_l & 0x0F) == 0x0F);
                    }
                    R8::HLIndirect => {
                        let addr = self.registers.hl.get();
                        let old = self.memory.get(addr);
                        self.memory.inc(addr);
                        let new = self.memory.get(addr);

                        self.registers.af.set_zero(new == 0);
                        self.registers.af.set_subtract(false);
                        self.registers.af.set_half_carry((old & 0x0F) == 0x0F);
                    }
                    R8::A => {
                        let old_a = self.registers.af.hi();
                        self.registers.af.inc_hi();
                        let new_a = self.registers.af.hi();
            
                        self.registers.af.set_zero(new_a == 0);
                        self.registers.af.set_subtract(false);
                        self.registers.af.set_half_carry((old_a & 0x0F) == 0x0F);
                    }
                }

                self.inc_pc(1);
                if r8 == R8::HLIndirect { 12 } else { 4 }
            },
            Instruction::DecR8(r8) => {
                match r8 {
                    R8::B => self.registers.bc.dec_hi(),
                    R8::C => self.registers.bc.dec_lo(),
                    R8::D => self.registers.de.dec_hi(),
                    R8::E => self.registers.de.dec_lo(),
                    R8::H => self.registers.hl.dec_hi(),
                    R8::L => self.registers.hl.dec_lo(),
                    R8::HLIndirect => {
                        let addr = self.registers.hl.get();
                        self.memory.dec(addr);
                    }
                    R8::A => self.registers.af.dec_hi(),
                }

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
                let mut condition = false;

                match cond {
                    Cond::Nz => condition = self.registers.af.zero() == false,
                    Cond::Z => condition = self.registers.af.zero() == true,
                    Cond::Nc => condition = self.registers.af.carry() == false,
                    Cond::C => condition = self.registers.af.carry() == true,
                }

                if condition {
                    let imm8 = self.fetch_imm8() as i8;

                    self.inc_pc(2);
                    let new_pc = self.registers.pc.get().wrapping_add_signed(imm8 as i16);
                    self.registers.pc.set(new_pc);
                    12
                } else {
                    self.inc_pc(2);
                    8
                }
            },

            Instruction::Stop => {
                // TODO
                4
            }



            Instruction::LdR8R8(lr8, rr8) => {
                if lr8 == R8::HLIndirect || rr8 == R8::HLIndirect { 8 } else { 4 }
            },
            // 1
            Instruction::Halt => {
                4
            },



            Instruction::AddAR8(r8) => {
                if r8 == R8::HLIndirect { 8 } else { 4 }
            },
            Instruction::AdcAR8(r8) => {
                if r8 == R8::HLIndirect { 8 } else { 4 }
            },
            Instruction::SubAR8(r8) => {
                if r8 == R8::HLIndirect { 8 } else { 4 }
            },
            Instruction::SbcAR8(r8) => {
                if r8 == R8::HLIndirect { 8 } else { 4 }
            },
            Instruction::AndAR8(r8) => {
                if r8 == R8::HLIndirect { 8 } else { 4 }
            },
            Instruction::XorAR8(r8) => {
                if r8 == R8::HLIndirect { 8 } else { 4 }
            },
            Instruction::OrAR8(r8) => {
                if r8 == R8::HLIndirect { 8 } else { 4 }
            },
            Instruction::CpAR8(r8) => {
                if r8 == R8::HLIndirect { 8 } else { 4 }
            },



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
