pub enum Interrupt {
    VBlank,
    Lcd,
    Timer,
    Serial,
    Joypad,
}

impl Interrupt {
    pub fn address(&self) -> u16 {
        match self {
            Interrupt::VBlank => 0x0040,
            Interrupt::Lcd => 0x0048,
            Interrupt::Timer => 0x0050,
            Interrupt::Serial => 0x0058,
            Interrupt::Joypad => 0x0060,
        }
    }

    fn bit_mask(&self) -> u8 {
        match self {
            Interrupt::VBlank => 0b0000_0001,
            Interrupt::Lcd => 0b0000_0010,
            Interrupt::Timer => 0b0000_0100,
            Interrupt::Serial => 0b0000_1000,
            Interrupt::Joypad => 0b0001_0000,
        }
    }
}

pub struct Memory([u8; 65536]);

impl Memory {
    pub fn new() -> Self {
        Memory([0; 65536])
    }

    pub fn get(&self, addr: u16) -> u8 {
        self.0[addr as usize]
    }

    pub fn set(&mut self, addr: u16, value: u8) {
        self.0[addr as usize] = value
    }

    pub fn inc(&mut self, addr: u16) {
        self.0[addr as usize] = self.0[addr as usize].wrapping_add(1);
    }

    pub fn dec(&mut self, addr: u16) {
        self.0[addr as usize] = self.0[addr as usize].wrapping_sub(1);
    }

    // interrupt flag
    pub fn is_if_set(&self, interrupt: Interrupt) -> bool {
        self.0[0xFF0F] & interrupt.bit_mask() != 0
    }
    
    pub fn set_if(&mut self, interrupt: Interrupt) {
        self.0[0xFF0F] |= interrupt.bit_mask();
    }

    pub fn unset_if(&mut self, interrupt: Interrupt) {
        self.0[0xFF0F] &= !interrupt.bit_mask();
    }

    pub fn get_interrupt(&self) -> Option<Interrupt> {
        // get the highest priority interrupt if an interrupt is enabled and set
        let ie = self.0[0xFFFF];
        let if_ = self.0[0xFF0F];
        let interrupt_priority = [
            Interrupt::VBlank,
            Interrupt::Lcd,
            Interrupt::Timer,
            Interrupt::Serial,
            Interrupt::Joypad,
        ];

        interrupt_priority.into_iter().find(|interrupt| (ie & interrupt.bit_mask() != 0) && (if_ & interrupt.bit_mask() != 0))
    }
}
