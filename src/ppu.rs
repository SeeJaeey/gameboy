use crate::ppu::ScanlineMode::{Drawing, HBlank, OAMScan, VBlank};

const DISPLAY_WIDTH: usize = 160;
const DISPLAY_HEIGHT: usize = 144;

const LCDC: u16 = 0xFF40;
const STAT: u16 = 0xFF41;
const SCY: u16 = 0xFF42;
const SCX: u16 = 0xFF43;
const LY: u16 = 0xFF44;
const LYC: u16 = 0xFF45;
const BGP: u16 = 0xFF47;
const OBP0: u16 = 0xFF48;
const OBP1: u16 = 0xFF49;
const WY: u16 = 0xFF4A;
const WX: u16 = 0xFF4B;

const TILE_DATA_START: u16 = 0x8000;
const TILE_DATA_END: u16 = 0x97FF;

const OAM_START: u16 = 0xFE00;
const OAM_END: u16 = 0xFE9F;
const VRAM_START: u16 = 0x8000;
const VRAM_END: u16 = 0x9FFF;

const VRAM_SIZE: usize = 8192;
const OAM_SIZE: usize = 160;

const CYCLES_PER_FRAME: u64 = 70224;
const CYCLES_PER_SCANLINE: u64 = 456;

#[derive(PartialEq)]
enum ScanlineMode {
    HBlank,
    VBlank,
    OAMScan,
    Drawing,
}

struct PpuRegister {
    Lcdc: u8,
    Stat: u8,
    Scy: u8,
    Scx: u8,
    Ly: u8,
    Lyc: u8,
    Bgp: u8,
    Obp0: u8,
    Obp1: u8,
    Wy: u8,
    Wx: u8,
}

impl PpuRegister {
    fn new() -> Self {
        Self {
            Lcdc: 0,
            Stat: 0,
            Scy: 0,
            Scx: 0,
            Ly: 0,
            Lyc: 0,
            Bgp: 0,
            Obp0: 0,
            Obp1: 0,
            Wy: 0,
            Wx: 0,
        }
    }

    fn get(&self, addr: u16) -> u8 {
        match addr {
            0xFF40 => self.Lcdc,
            0xFF41 => self.Stat,
            0xFF42 => self.Scy,
            0xFF43 => self.Scx,
            0xFF44 => self.Ly,
            0xFF45 => self.Lyc,
            0xFF47 => self.Bgp,
            0xFF48 => self.Obp0,
            0xFF49 => self.Obp1,
            0xFF4A => self.Wy,
            0xFF4B => self.Wx,
            _ => unreachable!(),
        }
    }

    fn set(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF40 => self.Lcdc = value,
            0xFF41 => self.Stat = value,
            0xFF42 => self.Scy = value,
            0xFF43 => self.Scx = value,
            0xFF44 => self.Ly = value,
            0xFF45 => self.Lyc = value,
            0xFF47 => self.Bgp = value,
            0xFF48 => self.Obp0 = value,
            0xFF49 => self.Obp1 = value,
            0xFF4A => self.Wy = value,
            0xFF4B => self.Wx = value,
            _ => unreachable!(),
        }
    }
}

pub struct PixelProcessingUnit {
    frame_buffer: [u8; DISPLAY_WIDTH * DISPLAY_HEIGHT],
    registers: PpuRegister,
    vram: [u8; VRAM_SIZE],
    oam: [u8; OAM_SIZE],
    internal_counter: u64,
    mode: ScanlineMode,
    old_mode: ScanlineMode,
}

impl PixelProcessingUnit {
    pub fn new() -> Self {
        Self {
            frame_buffer: [0; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            registers: PpuRegister::new(),
            vram: [0; VRAM_SIZE],
            oam: [0; OAM_SIZE],
            internal_counter: 0,
            mode: ScanlineMode::HBlank,
            old_mode: ScanlineMode::HBlank,
        }

    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            OAM_START..=OAM_END => self.oam[(addr - OAM_START) as usize],
            VRAM_START..=VRAM_END => self.vram[(addr - VRAM_START) as usize],
            0xFF40..=0xFF4B => self.registers.get(addr),
            _ => unreachable!(),
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            OAM_START..=OAM_END => {
                self.oam[(addr - OAM_START) as usize] = value;
            }
            VRAM_START..=VRAM_END =>  {
                self.vram[(addr - VRAM_START) as usize] = value;
            }
            0xFF40..=0xFF4B => self.registers.set(addr, value),
            _ => unreachable!(),
        }
    }

    pub fn tick(&mut self, cycles: u8) {
        let old_counter = self.internal_counter;
        self.internal_counter += cycles as u64;

        let frame_cycles = self.internal_counter % CYCLES_PER_FRAME;
        let line = frame_cycles / CYCLES_PER_SCANLINE;
        let position = frame_cycles % CYCLES_PER_SCANLINE;

        if line >= 144 {
            self.mode = VBlank;

            if self.old_mode != VBlank {
                // TODO: set interrupt flag (or return it in this function)
            }
        }

        // TODO: Wie finde ich Drawing und HBlank heraus? können variieren, OAMScan ist immer 80 cycles lang
        match position {
            0..=80 => {
                self.mode = OAMScan;
            }
            81..=252 => {
                self.mode = Drawing;
            }
            _ => {
                self.mode = HBlank;
            }
        }
    }
}