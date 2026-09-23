pub trait MemoryBankControllerT {
    fn from_header(rom_bytes: &Vec<u8>) -> Self;
    fn read(&self, rom: &[u8], ram: &[u8], addr: u16) -> u8;
    fn write(&mut self, rom: &[u8], ram: &mut [u8], addr: u16, value: u8);
}

pub struct RomOnly;

impl MemoryBankControllerT for RomOnly {
    fn from_header(rom_bytes: &Vec<u8>) -> Self {
        RomOnly
    }

    fn read(&self, rom: &[u8], _ram: &[u8], addr: u16) -> u8 {
        rom[addr as usize]
    }

    fn write(&mut self, _rom: &[u8], _ram: &mut [u8], _addr: u16, _value: u8) {
        // do nothing for rom only
    }
}

pub struct Mbc1 {
    rom_bank: u8,
    ram_bank: u8,
    ram_enabled: bool,
    banking_mode: u8,
}

// TODO: finish implementing Mbc1
impl MemoryBankControllerT for Mbc1 {
    fn from_header(rom_bytes: &Vec<u8>) -> Self {
        Mbc1 { rom_bank: 0, ram_bank: 0, ram_enabled: false, banking_mode: 0 }
    }

    fn read(&self, rom: &[u8], ram: &[u8], addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => rom[addr as usize],
            0x4000..=0x7FFF => {
                let bank = self.rom_bank.max(1) as usize;
                rom[bank * 0x4000 + (addr as usize - 0x4000)]
            }
            0xA000..=0xBFFF if self.ram_enabled => {
                ram[self.ram_bank as usize * 0x2000 + (addr as usize - 0xA000)]
            }
            _ => 0xFF,
        }
    }

    fn write(&mut self, _rom: &[u8], ram: &mut [u8], addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram_enabled = value & 0x0F == 0x0A,
            0x2000..=0x3FFF => self.rom_bank = value & 0x1F,
            _ => {}
        }
    }
}