use std::{env, fs};

use gameboy::{cartridge::Cartridge, emulator::Emulator};
use minifb::{Key, Scale, Window, WindowOptions};

const WIDTH: usize = 160;
const HEIGHT: usize = 144;
const CYCLES_PER_FRAME: u32 = 70224;

const PALETTE: [u32; 4] = [0xE0F8D0, 0x88C070, 0x346856, 0x081820];

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("You need to specify a rom path.")
    }

    let rom_bytes = fs::read(&args[1]).expect("Could not read rom.");
    let mut emulator = Emulator::new_with_cartridge(Cartridge::new(rom_bytes));

    let mut window = Window::new(
        "Game Boy",
        WIDTH,
        HEIGHT,
        WindowOptions { scale: Scale::X4, ..WindowOptions::default() },
    )
    .expect("Could not create window.");
    window.set_target_fps(60);

    let mut pixels = vec![0u32; WIDTH * HEIGHT];
    let mut cycles_this_frame: u32 = 0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        cycles_this_frame += emulator.step() as u32;

        if emulator.memory.take_frame_ready() || cycles_this_frame >= CYCLES_PER_FRAME {
            cycles_this_frame = 0;

            for (dst, &shade) in pixels.iter_mut().zip(emulator.memory.frame_buffer()) {
                *dst = PALETTE[(shade & 0b11) as usize];
            }
            window
                .update_with_buffer(&pixels, WIDTH, HEIGHT)
                .expect("Could not update window.");
        }
    }
}
