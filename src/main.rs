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

    let mut emulator = Emulator::new_with_cartridge(cartridge);

    let mut last_pc = emulator.registers.pc.get();
    let mut same_pc_count = 0;

    loop {
        emulator.step();

        let pc = emulator.registers.pc.get();

        if pc == last_pc {
            same_pc_count += 1;
            if same_pc_count > 100 {
                break;
            }
        } else {
            same_pc_count = 0;
        }
        last_pc = pc;
    }
}
