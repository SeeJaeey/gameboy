use crate::cpu::{Registers, Register16};
use crate::decode::{Instruction, R8, R16, R16Mem, R16Stk};
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
                self.registers.pc.set(self.registers.pc.get() + 3);
                8
            },
            Instruction::LdR16memA(r16Mem) => {
                8
            },
            Instruction::LdAR16mem(r16Mem) => {
                8
            },
            Instruction::LdImm16Sp => {
                20
            },

            Instruction::IncR16(r16) => {
                8
            },
            Instruction::DecR16(r16) => {
                8
            },
            Instruction::AddHlR16(r16) => {
                8
            },

            Instruction::IncR8(r8) => {
                if r8 == R8::HLIndirect { 12 } else { 4 }
            },
            Instruction::DecR8(r8) => {
                if r8 == R8::HLIndirect { 12 } else { 4 }
            },

            Instruction::LdR8Imm8(r8) => {
                if r8 == R8::HLIndirect { 12 } else { 8 }
            },

            Instruction::Rlca => {
                4
            },
            Instruction::Rrca => {
                4
            },
            Instruction::Rla => {
                4
            },
            Instruction::Rra => {
                4
            },
            Instruction::Daa => {
                4
            },
            Instruction::Cpl => {
                4
            },
            Instruction::Scf => {
                4
            },
            Instruction::Ccf => {
                4
            },

            Instruction::JrImm8 => {
                12
            },
            Instruction::JrCondImm8(cond) => {
                12 // 12/8
            },

            Instruction::Stop => {
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

    pub fn fetch_imm8(&self) -> u8 {
        self.memory.0[(self.registers.pc.get() + 1) as usize]
    }

    pub fn fetch_imm16(&self) -> u16 {
        let lo = self.memory.0[(self.registers.pc.get() + 1) as usize] as u16;
        let hi = self.memory.0[(self.registers.pc.get() + 2) as usize] as u16;
        (hi << 8) | lo
    }
}
