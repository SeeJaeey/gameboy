const FLAG_ZERO: u8 = 0b1000_0000;
const FLAG_SUBTRACT: u8 = 0b0100_0000;
const FLAG_HALF_CARRY: u8 = 0b0010_0000;
const FLAG_CARRY: u8 = 0b0001_0000;

#[derive(Copy, Clone)]
pub struct Register16(u16);

impl Register16 {
    pub fn new(value: u16) -> Self {
        Register16(value)
    }

    pub fn get(&self) -> u16 {
        self.0
    }

    pub fn set(&mut self, value: u16) {
        self.0 = value;
    }

    pub fn hi(&self) -> u8 {
        (self.0 >> 8) as u8
    }

    pub fn lo(&self) -> u8 {
        (self.0 & 0x00FF) as u8
    }

    pub fn set_hi(&mut self, value: u8) {
        self.0 = (self.0 & 0x00FF) | ((value as u16) << 8);
    }

    pub fn set_lo(&mut self, value: u8) {
        self.0 = (self.0 & 0xFF00) | (value as u16);
    }

    pub fn inc(&mut self) {
        self.0 = self.0.wrapping_add(1);
    }

    pub fn inc_hi(&mut self) {
        self.set_hi(self.hi().wrapping_add(1));
    }

    pub fn inc_lo(&mut self) {
        self.set_lo(self.lo().wrapping_add(1));
    }

    pub fn dec(&mut self) {
        self.0 = self.0.wrapping_sub(1);
    }

    pub fn dec_hi(&mut self) {
        self.set_hi(self.hi().wrapping_sub(1));
    }

    pub fn dec_lo(&mut self) {
        self.set_lo(self.lo().wrapping_sub(1));
    }

    pub fn zero(&self) -> bool {
        self.lo() & FLAG_ZERO != 0
    }
    pub fn set_zero(&mut self, v: bool) {
        self.set_flag_bit(FLAG_ZERO, v)
    }

    pub fn subtract(&self) -> bool {
        self.lo() & FLAG_SUBTRACT != 0
    }
    pub fn set_subtract(&mut self, v: bool) {
        self.set_flag_bit(FLAG_SUBTRACT, v)
    }

    pub fn half_carry(&self) -> bool {
        self.lo() & FLAG_HALF_CARRY != 0
    }
    pub fn set_half_carry(&mut self, v: bool) {
        self.set_flag_bit(FLAG_HALF_CARRY, v)
    }

    pub fn carry(&self) -> bool {
        self.lo() & FLAG_CARRY != 0
    }
    pub fn set_carry(&mut self, v: bool) {
        self.set_flag_bit(FLAG_CARRY, v)
    }

    fn set_flag_bit(&mut self, mask: u8, value: bool) {
        let f = if value {
            self.lo() | mask
        } else {
            self.lo() & !mask
        };
        self.set_lo(f);
    }

    pub fn set_flags(&mut self, z: bool, n: bool, h: bool, c: bool) {
        let mut f = 0u8;
        if z {
            f |= FLAG_ZERO;
        }
        if n {
            f |= FLAG_SUBTRACT;
        }
        if h {
            f |= FLAG_HALF_CARRY;
        }
        if c {
            f |= FLAG_CARRY;
        }
        self.set_lo(f);
    }
}

pub struct Flags {
    pub zero: bool,
    pub subtract: bool,
    pub half_carry: bool,
    pub carry: bool,
}

impl Flags {
    pub fn from_af(af: &Register16) -> Self {
        let bits = af.lo();
        Flags {
            zero: bits & FLAG_ZERO == FLAG_ZERO,
            subtract: bits & FLAG_SUBTRACT == FLAG_SUBTRACT,
            half_carry: bits & FLAG_HALF_CARRY == FLAG_HALF_CARRY,
            carry: bits & FLAG_CARRY == FLAG_CARRY,
        }
    }
}

pub struct Registers {
    pub af: Register16,
    pub bc: Register16,
    pub de: Register16,
    pub hl: Register16,
    pub sp: Register16,
    pub pc: Register16,
}

impl Registers {
    pub fn new() -> Self {
        Registers {
            af: Register16::new(0),
            bc: Register16::new(0),
            de: Register16::new(0),
            hl: Register16::new(0),
            sp: Register16::new(0),
            pc: Register16::new(0),
        }
    }
}
