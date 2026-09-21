const DIV_ADDR: u16 = 0xFF04;
const TIMA_ADDR: u16 = 0xFF05;
const TMA_ADDR: u16 = 0xFF06;
const TAC_ADDR: u16 = 0xFF07;

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

pub struct Memory {
    data: [u8; 65536],
    internal_counter: u16,
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            data: [0; 65536],
            internal_counter: 0,
        }
    }

    pub fn get(&self, addr: u16) -> u8 {
        self.data[addr as usize]
    }

    pub fn set(&mut self, addr: u16, value: u8) {
        if addr == DIV_ADDR {
            self.data[addr as usize] = 0;
            self.internal_counter = 0;
        } else {
            self.data[addr as usize] = value
        }
    }

    pub fn inc(&mut self, addr: u16) {
        self.data[addr as usize] = self.data[addr as usize].wrapping_add(1);
    }

    pub fn dec(&mut self, addr: u16) {
        self.data[addr as usize] = self.data[addr as usize].wrapping_sub(1);
    }

    // interrupt flag
    pub fn is_if_set(&self, interrupt: Interrupt) -> bool {
        self.data[0xFF0F] & interrupt.bit_mask() != 0
    }
    
    pub fn set_if(&mut self, interrupt: Interrupt) {
        self.data[0xFF0F] |= interrupt.bit_mask();
    }

    pub fn unset_if(&mut self, interrupt: Interrupt) {
        self.data[0xFF0F] &= !interrupt.bit_mask();
    }

    pub fn get_interrupt(&self) -> Option<Interrupt> {
        // get the highest priority interrupt if an interrupt is enabled and set
        let ie = self.data[0xFFFF];
        let if_ = self.data[0xFF0F];
        let interrupt_priority = [
            Interrupt::VBlank,
            Interrupt::Lcd,
            Interrupt::Timer,
            Interrupt::Serial,
            Interrupt::Joypad,
        ];

        interrupt_priority.into_iter().find(|interrupt| (ie & interrupt.bit_mask() != 0) && (if_ & interrupt.bit_mask() != 0))
    }

    pub fn tick(&mut self, cycles: u8) {
        let tac = self.get(TAC_ADDR);
        let tima_freq = tac & 0b0000_0011;
        let tima_enabled = tac & 0b0000_0100 != 0;
        let tima_bit_mask = match tima_freq {
            0b00 => 0b0000_0010_0000_0000,
            0b01 => 0b0000_0000_0000_1000,
            0b10 => 0b0000_0000_0010_0000,
            0b11 => 0b0000_0000_1000_0000,
            _ => unreachable!(),
        };

        let mut tima_increments: u8 = 0;

        for _ in 0..cycles {
            let tima_bit_before = self.internal_counter & tima_bit_mask;
            self.internal_counter = self.internal_counter.wrapping_add(1);
            let tima_bit_after = self.internal_counter & tima_bit_mask;

            if tima_bit_before != 0 && tima_bit_after == 0 {
                tima_increments += 1;
            }
        }

        let div_val = (self.internal_counter >> 8) as u8;
        self.data[DIV_ADDR as usize] = div_val;

        if !tima_enabled {
            return;
        }

        for _ in 0..tima_increments {
            let tima_val = self.get(TIMA_ADDR).wrapping_add(1);

            if tima_val == 0 {
                self.data[TIMA_ADDR as usize] = self.get(TMA_ADDR);
                self.set_if(Interrupt::Timer);
            } else {
                self.data[TIMA_ADDR as usize] = tima_val;
            }
        }
    }
}
