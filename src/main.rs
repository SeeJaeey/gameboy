use std::{env, fs};

use gameboy::{cartridge::Cartridge, emulator::Emulator};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("You need to specify a rom path.")
    }

    let rom_path = &args[1];
    let result = fs::read(rom_path);
    let rom_bytes = result.expect("Could not read rom.");

    let cartridge = Cartridge::new(rom_bytes);

    let emulator = Emulator::new_with_cartridge(cartridge);

    // TODO: step loop
}
