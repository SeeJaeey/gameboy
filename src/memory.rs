pub struct Memory (pub [u8; 65536]);

impl Memory {
    pub fn new () -> Self {
        Memory ([0; 65536])
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
}
