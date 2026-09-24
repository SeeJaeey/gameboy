const EXTERNAL_RAM_ADDRESS_START: u16 = 0xA000;
const EXTERNAL_RAM_ADDRESS_END: u16 = 0xBFFF;
const ROM_BANK_0_ADDRESS_START: u16 = 0x0000;
const ROM_BANK_0_ADDRESS_END: u16 = 0x3FFF;
const ROM_BANK_N_ADDRESS_START: u16 = 0x4000;
const ROM_BANK_N_ADDRESS_END: u16 = 0x7FFF;

const RAM_BANK_SIZE: usize = 0x2000;
const ROM_BANK_SIZE: usize = 0x4000;

pub trait MemoryBankControllerT {
    fn from_header(rom_bytes: &Vec<u8>) -> Self;
    fn read(&self, rom: &[u8], ram: &[u8], addr: u16) -> u8;
    fn write(&mut self, rom: &[u8], ram: &mut [u8], addr: u16, value: u8);
}

pub struct RomOnly;

impl MemoryBankControllerT for RomOnly {
    fn from_header(_rom_bytes: &Vec<u8>) -> Self {
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

impl MemoryBankControllerT for Mbc1 {
    fn from_header(_rom_bytes: &Vec<u8>) -> Self {
        Mbc1 { rom_bank: 0, ram_bank: 0, ram_enabled: false, banking_mode: 0 }
    }

    fn read(&self, rom: &[u8], ram: &[u8], addr: u16) -> u8 {
        match addr {
            ROM_BANK_0_ADDRESS_START..=ROM_BANK_0_ADDRESS_END => rom[addr as usize],
            ROM_BANK_N_ADDRESS_START..=ROM_BANK_N_ADDRESS_END => {
                let bank = (self.ram_bank << 5) | self.rom_bank.max(1);
                let num_banks = (rom.len() / ROM_BANK_SIZE).max(1);
                let rom_bank = bank as usize % num_banks;

                rom[rom_bank * ROM_BANK_SIZE + (addr - ROM_BANK_N_ADDRESS_START) as usize]
            }
            EXTERNAL_RAM_ADDRESS_START..=EXTERNAL_RAM_ADDRESS_END if self.ram_enabled => {
                let bank = if self.banking_mode == 1 { self.ram_bank } else { 0 };
                let num_banks = (ram.len() / RAM_BANK_SIZE).max(1);
                let ram_bank = bank as usize % num_banks;

                ram[ram_bank as usize * RAM_BANK_SIZE + (addr - EXTERNAL_RAM_ADDRESS_START) as usize]
            }
            _ => 0xFF,
        }
    }

    fn write(&mut self, _rom: &[u8], ram: &mut [u8], addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram_enabled = value & 0b1111 == 0b1010,
            0x2000..=0x3FFF => self.rom_bank = value & 0b11111,
            0x4000..=0x5FFF => self.ram_bank = value & 0b11,
            0x6000..=0x7FFF => self.banking_mode = value & 0b1,
            EXTERNAL_RAM_ADDRESS_START..=EXTERNAL_RAM_ADDRESS_END if self.ram_enabled => {
                let bank = if self.banking_mode == 1 { self.ram_bank } else { 0 };
                let num_banks = (ram.len() / RAM_BANK_SIZE).max(1);
                let ram_bank = bank as usize % num_banks;

                ram[ram_bank as usize * RAM_BANK_SIZE + (addr - EXTERNAL_RAM_ADDRESS_START) as usize] = value;
            }
            _ => {}
        }
    }
}