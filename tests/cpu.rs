
use std::fs::File;
use gameboy::emulator::Emulator;
use serde::{Deserialize};

#[derive(Deserialize)]
struct CpuTest {
    name: String,
    initial: CpuState,

    #[serde(rename = "final")]
    final_: CpuState,
    // cycles: Vec<Option<Cycle>>,
}

#[derive(Deserialize)]
struct CpuState {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
    pc: u16,
    sp: u16,
    ram: Vec<(u16, u8)>,
}

fn run_test(test: &CpuTest) -> Result<(), String> {
    let mut emulator = Emulator::new();

    apply_initial_state(&mut emulator, &test.initial);

    emulator.step();

    assert_final_state(&emulator, &test.final_)
}

fn apply_initial_state(emulator: &mut Emulator, state: &CpuState) {
    emulator.registers.af.set(((state.a as u16) << 8) | (state.f as u16));
    emulator.registers.bc.set(((state.b as u16) << 8) | (state.c as u16));
    emulator.registers.de.set(((state.d as u16) << 8) | (state.e as u16));
    emulator.registers.hl.set(((state.h as u16) << 8) | (state.l as u16));
    emulator.registers.pc.set(state.pc.wrapping_sub(1)); // test data assumes fetch already happened
    emulator.registers.sp.set(state.sp);

    for (addr, value) in &state.ram {
        emulator.memory.set(*addr, *value);
    }
}

fn assert_final_state(emulator: &Emulator, state: &CpuState) -> Result<(), String> {
    let mut errors = Vec::new();

    let actual_a = emulator.registers.af.hi();
    if actual_a != state.a {
        errors.push(format!("A: expected {}, got {}", state.a, actual_a));
    }

    let actual_f = emulator.registers.af.lo();
    if actual_f != state.f {
        errors.push(format!("F: expected {}, got {}", state.f, actual_f));
    }

    let actual_b = emulator.registers.bc.hi();
    if actual_b != state.b {
        errors.push(format!("B: expected {}, got {}", state.b, actual_b));
    }

    let actual_c = emulator.registers.bc.lo();
    if actual_c != state.c {
        errors.push(format!("C: expected {}, got {}", state.c, actual_c));
    }

    let actual_d = emulator.registers.de.hi();
    if actual_d != state.d {
        errors.push(format!("D: expected {}, got {}", state.d, actual_d));
    }

    let actual_e = emulator.registers.de.lo();
    if actual_e != state.e {
        errors.push(format!("E: expected {}, got {}", state.e, actual_e));
    }

    let actual_h = emulator.registers.hl.hi();
    if actual_h != state.h {
        errors.push(format!("H: expected {}, got {}", state.h, actual_h));
    }

    let actual_l = emulator.registers.hl.lo();
    if actual_l != state.l {
        errors.push(format!("L: expected {}, got {}", state.l, actual_l));
    }

    let actual_pc = emulator.registers.pc.get();
    let expected_pc = state.pc.wrapping_sub(1); // test data assumes additional prefetch step
    if actual_pc != expected_pc {
        errors.push(format!("PC: expected {}, got {}", state.pc, actual_pc));
    }

    let actual_sp = emulator.registers.sp.get();
    if actual_sp != state.sp {
        errors.push(format!("SP: expected {}, got {}", state.sp, actual_sp));
    }

    for (addr, expected) in &state.ram {
        let actual = emulator.memory.get(*addr);

        if actual != *expected {
            errors.push(format!(
                "RAM[0x{:04X}]: expected 0x{:02X}, got 0x{:02X}",
                addr, expected, actual
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

#[test]
fn cpu_tests() {
    let data_dir = "tests/data/cpu";

    let mut entries: Vec<_> = std::fs::read_dir(data_dir)
        .unwrap_or_else(|_| panic!("Could not read test data dir: {}", data_dir))
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|ext| ext.to_str()) == Some("json"))
        .collect();
    entries.sort_by_key(|e| e.path()); // sort opcodes

    let mut failures = Vec::new();
    let mut total = 0;

    for entry in entries {
        let path = entry.path();
        //println!("Starting test: {}", path.display());

        let file = File::open(&path)
            .unwrap_or_else(|_| panic!("Could not open cpu test data: {:?}", path));

        let cpu_tests: Vec<CpuTest> = serde_json::from_reader(file)
            .unwrap_or_else(|_| panic!("Could not deserialize cpu test data: {:?}", path));

        for test in &cpu_tests {
            total += 1;

            if let Err(error) = run_test(test) {
                failures.push(format!("[{}] '{}': {}", path.display(), test.name, error));
            }
        }
    }

    if !failures.is_empty() {
        panic!(
            "{} of {} CPU tests failed:\n{}",
            failures.len(),
            total,
            failures.join("\n")
        );
    }
}

