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
