#[macro_use]
extern crate log;

use std::fs::File;
use sdl2::pixels::PixelFormatEnum;
use std::time::Instant;

fn main() {
    simple_logger::init_by_env();

    let args = std::env::args().collect::<Vec<String>>();
    if args.len() != 2 {
        eprintln!("Usage: {} <program.8xp>", args[0]);
        return;
    }

    let sdl_context = sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();
    let scale = 4;
    let mut window = video
        .window("tihle", 96 * scale, 64 * scale)
        .build()
        .unwrap()
        .into_canvas()
        .software()
        .build()
        .unwrap();

    let texture_creator = window.texture_creator();
    let display_tex = texture_creator.create_texture_streaming(PixelFormatEnum::RGB24, 96, 64).unwrap();

    let mut events = sdl_context.event_pump().unwrap();

    let mut emu = tihle::Emulator::new();
    let mut cpu = tihle::Z80::new();
    emu.load_program(
        &mut cpu,
        File::open(&args[1]).expect("Failed to open program file"),
    )
    .expect("Failed to load program");

    trace!("Entering run loop. {:#?}", cpu.regs());
    'runloop: loop {
        let frame_start = Instant::now();

        // Process events
        for event in events.poll_iter() {
            use sdl2::event::Event;

            match event {
                Event::KeyDown {
                    keycode: Some(k),
                    repeat: false,
                    ..
                } => {
                    debug!("key down: {}", k);
                },
                Event::DropFile {
                    filename, ..
                } => {
                    emu.reset();
                    match File::open(filename) {
                        Ok(f) => {
                            if let Err(e) = emu.load_program(&mut cpu, f) {
                                error!("Failed to load program: {:?}", e);
                            }
                        },
                        Err(e) => {
                            error!("Unable to open file to load: {}", e);
                        }
                    }
                },
                Event::Quit { .. } => {
                    break 'runloop;
                }
                _ => {}
            }
        }

        emu.run(&mut cpu, &frame_start);

        window.copy(&display_tex, None, None).unwrap();
    }
}
