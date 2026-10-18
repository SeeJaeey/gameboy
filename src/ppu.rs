use crate::ppu::{ScanlineMode::{Drawing, HBlank, OAMScan, VBlank}};

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

const CYCLES_PER_FRAME: u32 = 70224;
const CYCLES_PER_SCANLINE: u32 = 456;

#[derive(PartialEq, Copy, Clone)]
enum ScanlineMode {
    HBlank,
    VBlank,
    OAMScan,
    Drawing,
}

pub struct PpuInfo {
    pub frame_ready: bool,
    pub lcd_interrupt: bool,
    pub vblank_interrupt: bool,
}

impl PpuInfo {
    pub fn new() -> Self {
        PpuInfo { frame_ready: false, lcd_interrupt: false, vblank_interrupt: false }
    }
}

struct PpuRegister {
    lcdc: u8,
    stat: u8,
    scy: u8,
    scx: u8,
    ly: u8,
    lyc: u8,
    bgp: u8,
    obp0: u8,
    obp1: u8,
    wy: u8,
    wx: u8,
}

impl PpuRegister {
    fn new() -> Self {
        Self {
            lcdc: 0x91,
            stat: 0x80,
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            bgp: 0xFC,
            obp0: 0xFF,
            obp1: 0xFF,
            wy: 0,
            wx: 0,
        }
    }

    fn get(&self, addr: u16) -> u8 {
        match addr {
            0xFF40 => self.lcdc,
            0xFF41 => self.stat,
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            0xFF47 => self.bgp,
            0xFF48 => self.obp0,
            0xFF49 => self.obp1,
            0xFF4A => self.wy,
            0xFF4B => self.wx,
            _ => unreachable!(),
        }
    }

    fn set(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF40 => self.lcdc = value,
            0xFF41 => self.stat = (self.stat & 0x87) | (value & 0x78), // some parts are read only for CPU
            0xFF42 => self.scy = value,
            0xFF43 => self.scx = value,
            0xFF44 => {}, // read only for CPU
            0xFF45 => self.lyc = value,
            0xFF47 => self.bgp = value,
            0xFF48 => self.obp0 = value,
            0xFF49 => self.obp1 = value,
            0xFF4A => self.wy = value,
            0xFF4B => self.wx = value,
            _ => unreachable!(),
        }
    }
}

pub struct PixelProcessingUnit {
    frame_buffer: [u8; DISPLAY_WIDTH * DISPLAY_HEIGHT],
    registers: PpuRegister,
    vram: [u8; VRAM_SIZE],
    oam: [u8; OAM_SIZE],
    internal_counter: u32,
    mode: ScanlineMode,
    stat_line: bool,
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
            stat_line: false,
        }

    }

    pub fn frame_buffer(&self) -> &[u8] { &self.frame_buffer }

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

    pub fn tick(&mut self, cycles: u8) -> PpuInfo {
        let mut info = PpuInfo::new();

        if self.registers.lcdc & 0x80 == 0 {
            self.internal_counter = 0;
            self.mode = HBlank;
            self.registers.ly = 0;
            self.stat_line = false;
            self.update_stat();
            return info;
        }

        let old_mode = self.mode;

        self.internal_counter = (self.internal_counter + cycles as u32) % CYCLES_PER_FRAME;
        let line = (self.internal_counter / CYCLES_PER_SCANLINE) as u8;
        let position = self.internal_counter % CYCLES_PER_SCANLINE;

        self.registers.ly = line;
        self.mode = if line >= 144 {
            VBlank
        } else {
            match position {
                0..=79 => OAMScan,
                80..=251 => Drawing,
                _ => HBlank,
            }
        };
        
        if self.mode == VBlank && old_mode != VBlank {
            info.vblank_interrupt = true;
            info.frame_ready = true;
        }

        self.update_stat();

        let stat = self.registers.stat;
        let mode_source = match self.mode {
            HBlank => stat & 0x08 != 0,
            VBlank => stat & 0x10 != 0,
            OAMScan => stat & 0x20 != 0,
            Drawing => false,
        };
        let lyc_source = (stat & 0x44) == 0x44; // LY==LYC-Flag und Enable-Bit 6
        let stat_line = mode_source || lyc_source;

        if stat_line && !self.stat_line {
            info.lcd_interrupt = true;
        }
        self.stat_line = stat_line;

        info
    }

    fn update_stat(&mut self) {
        let mode_bits = match self.mode {
            HBlank => 0,
            VBlank => 1,
            OAMScan => 2,
            Drawing => 3,
        };
        let lyc_flag = (self.registers.ly == self.registers.lyc) as u8;
        self.registers.stat = (self.registers.stat & 0xF8) | (lyc_flag << 2) | mode_bits;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_frame_has_one_vblank_and_ly_wraps() {
        let mut ppu = PixelProcessingUnit::new();
        let mut vblanks = 0;
        let mut max_ly = 0;

        for _ in 0..(CYCLES_PER_FRAME / 4) {
            let info = ppu.tick(4);
            if info.vblank_interrupt {
                vblanks += 1;
            }
            max_ly = max_ly.max(ppu.read(LY));
        }

        assert_eq!(vblanks, 1);
        assert_eq!(max_ly, 153);
        assert_eq!(ppu.read(LY), 0);
    }
}