use std::fs::File;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() != 2 {
        eprintln!("Usage: {} <program.8xp>", args[0]);
        return;
    }

    let mut emu = tihle::Emulator::new();
    let mut cpu = tihle::Z80::new();
    emu.load_program(File::open(&args[1]).expect("Failed to open program file"))
        .expect("Failed to load program");

    emu.run(&mut cpu);
}
