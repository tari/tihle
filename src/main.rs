#[macro_use]
extern crate log;

use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::TextureCreator;
use sdl2::video::WindowContext;
use std::fs::File;
use std::time::Instant;
use tihle::display::Display;

const DISPLAY_SCALE: usize = 4;

struct Window {
    canvas: sdl2::render::WindowCanvas,
    texture_creator: TextureCreator<WindowContext>,
}

impl Window {
    fn create(subsystem: sdl2::VideoSubsystem) -> Self {
        let canvas = subsystem
            .window(
                "tihle",
                (Display::COLS * DISPLAY_SCALE) as u32,
                (Display::ROWS * DISPLAY_SCALE) as u32,
            )
            .build()
            .expect("Failed to build window for output")
            .into_canvas()
            .software()
            .build()
            .expect("Failed to build WindowCanvas");

        Self {
            texture_creator: canvas.texture_creator(),
            canvas,
        }
    }
}

/// Video output implementation.
///
/// We render the display to a texture, which gets rendered to a window with fixed size and
/// scale. We manually render index-1 LSb from the display driver to RGB24 because 1bpp is
/// not really supported by.. anything? Scaling to the full window is handled by SDL though.
struct Video<'a> {
    canvas: &'a mut sdl2::render::WindowCanvas,
    /// Rectangle for scaling display texture to window size.
    rect: Rect,
    /// The actual texture that we render to the window.
    texture: sdl2::render::Texture<'a>,
    /// Byte array to back texture updates.
    ///
    /// Read the emulator display and write to this, then update texture with the contents;
    /// necessary because we do a manual format conversion from the emulator format to the
    /// texture format.
    texture_buf: Box<[u8; Display::ROWS * Display::COLS * 3]>,
}

impl<'a> Video<'a> {
    fn setup(window: &'a mut Window) -> Self {
        let Window {
            ref mut texture_creator,
            ref mut canvas,
        } = window;

        let texture = texture_creator
            .create_texture_streaming(PixelFormatEnum::RGB24, 96, 64)
            .unwrap();
        let texture_buf = Box::new([0u8; Display::ROWS * Display::COLS * 3]);

        let rect = {
            let window_size = canvas.output_size().expect("Unable to get window size");
            Rect::new(0, 0, window_size.0, window_size.1)
        };

        Video {
            canvas,
            rect,
            texture,
            texture_buf,
        }
    }

    fn update(&mut self, display: &Display) {
        for (&src, dst) in display
            .get_buffer()
            .iter()
            .zip(self.texture_buf.chunks_exact_mut(3))
        {
            if src != 0 {
                dst.copy_from_slice(&[0, 0, 0]);
            } else {
                dst.copy_from_slice(&[0xFF, 0xFF, 0xFF]);
            }
        }
        self.texture
            .update(None, &self.texture_buf[..], Display::COLS * 3)
            .expect("Failed to update texture while rendering");

        self.canvas
            .copy(&self.texture, None, Some(self.rect))
            .expect("Failed to copy texture to window");
        self.canvas.present();
    }
}

fn main() {
    env_logger::init();

    let args = std::env::args().collect::<Vec<String>>();
    if args.len() != 2 {
        eprintln!("Usage: {} <program.8xp>", args[0]);
        return;
    }

    let sdl_context = sdl2::init().unwrap();

    let video_subsystem = sdl_context.video().unwrap();
    let mut window = Window::create(video_subsystem);
    let mut video = Video::setup(&mut window);

    let mut events = sdl_context.event_pump().unwrap();

    let mut emu = tihle::Emulator::new();
    let mut cpu = tihle::Z80::new();
    emu.load_program(
        &mut cpu,
        File::open(&args[1]).expect("Failed to open program file"),
    )
    .expect("Failed to load program");

    trace!("Entering run loop. {:#?}", cpu.regs_mut());
    'runloop: loop {
        let frame_start = Instant::now();

        video.update(&emu.display);

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
                }
                Event::DropFile { filename, .. } => {
                    emu.reset();
                    match File::open(filename) {
                        Ok(f) => {
                            if let Err(e) = emu.load_program(&mut cpu, f) {
                                error!("Failed to load program: {:?}", e);
                            }
                        }
                        Err(e) => {
                            error!("Unable to open file to load: {}", e);
                        }
                    }
                }
                Event::Quit { .. } => {
                    break 'runloop;
                }
                _ => {}
            }
        }

        if !emu.is_running() {
            break;
        }
        emu.run(&mut cpu, &frame_start);
    }
}
