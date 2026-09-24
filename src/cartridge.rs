use crate::mbc::{Mbc1, MemoryBankControllerT, RomOnly};

enum MemoryBankController {
    RomOnly(RomOnly),
    Mbc1(Mbc1),
    // Mbc2(Mbc2),
    // Mbc3(Mbc3),
    // Mbc5(Mbc5),
    // Mbc6(Mbc6),
}

impl MemoryBankController {
    pub fn read(&self, rom: &[u8], ram: &[u8], addr: u16) -> u8 {
        match self {
            MemoryBankController::RomOnly(ct) => ct.read(rom, ram, addr),
            MemoryBankController::Mbc1(ct) => ct.read(rom, ram, addr),
        }
    }
    
    pub fn write(&mut self, rom: &[u8], ram: &mut [u8], addr: u16, value: u8) {
        match self {
            MemoryBankController::RomOnly(ct) => ct.write(rom, ram, addr, value),
            MemoryBankController::Mbc1(ct) => ct.write(rom, ram, addr, value),
        }
    }
}

pub struct Cartridge {
    mbc: MemoryBankController,
    rom: Vec<u8>,
    ram: Vec<u8>,
}

impl Cartridge {
    pub fn new(rom: Vec<u8>) -> Self {
        let mbc = match rom.get(0x0147) {
            Some(0x00) => MemoryBankController::RomOnly(RomOnly),
            Some(0x01..=0x03) => MemoryBankController::Mbc1(Mbc1::from_header(&rom)),
            // TODO: add more cartridge types
            Some(t) => panic!("Unsupported cartridge type: {:#04x}", t),
            None => panic!("Invalid rom format, could not read rom type."),
        };

        let ram = match rom.get(0x0149) {
            Some(0x00) => vec![0; 0],
            Some(0x01) => vec![0; 2000],
            Some(0x02) => vec![0; 8000],
            Some(0x03) => vec![0; 32000],
            Some(0x04) => vec![0; 128000],
            Some(0x05) => vec![0; 64000],
            Some(s) => panic!("Unsupported ram size: {:#04x}", s),
            None => panic!("Invalid rom format, could not read ram size."),
        };

        Cartridge { mbc, rom, ram }
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.mbc.read(&self.rom, &self.ram, addr)
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        self.mbc.write(&self.rom, &mut self.ram, addr, value);
    }
}