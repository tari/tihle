#[macro_use]
extern crate log;

use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::TextureCreator;
use sdl2::video::WindowContext;
use std::ffi::c_void;
use std::fs::File;
use std::time::{Duration, Instant};
use tihle::{Display, Emulator, Z80};

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
            .accelerated()
            .build()
            .expect("Failed to build WindowCanvas");
        info!("Renderer set up: {:?}", canvas.info());

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
    ///
    /// The texture format used is YV12; since we really only need greyscale output, YV12
    /// is a good choice because it provides good locality for luminance (because it's planar
    /// rather than interleaved) and doesn't waste space on the chroma channels that we don't
    /// need. It's also widely supported by hardware accelerators.
    texture_buf: Box<
        [u8; (Display::ROWS * Display::COLS) + ((Display::ROWS / 2) * (Display::COLS / 2) * 2)],
    >,
}

impl<'a> Video<'a> {
    const PIXEL_FORMAT: PixelFormatEnum = PixelFormatEnum::YV12;

    fn setup(window: &'a mut Window) -> Self {
        let Window {
            ref mut texture_creator,
            ref mut canvas,
        } = window;

        let texture = texture_creator
            .create_texture_streaming(Self::PIXEL_FORMAT, 96, 64)
            .unwrap();
        let mut texture_buf = Box::new([0u8; 9216]);
        // Initialize chroma planes to neutral, and we won't touch them again
        for byte in texture_buf[Display::ROWS * Display::COLS..].iter_mut() {
            *byte = 128;
        }

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
        // Simple YV12 conversion: write luminance bytes and leave chroma untouched
        for (&src, dst) in display.get_buffer().iter().zip(self.texture_buf.iter_mut()) {
            if src != 0 {
                *dst = 0;
            } else {
                *dst = 0xFF;
            }
        }

        // PixelFormatEnum.byte_size* include all three planes so we don't have a convenient way
        // to ask what the pitch is.
        assert_eq!(
            Self::PIXEL_FORMAT,
            PixelFormatEnum::YV12,
            "Texture pitch may need to change with pixel format"
        );
        self.texture
            .update(None, &self.texture_buf[..], Display::COLS)
            .expect("Failed to update texture while rendering");

        self.canvas
            .copy(&self.texture, None, Some(self.rect))
            .expect("Failed to copy texture to window");
        self.canvas.present();
    }

    fn show_status(&mut self, status: &str) {
        self.canvas
            .window_mut()
            .set_title(&format!("tihle ({})", status))
            .unwrap();
    }
}

#[cfg(target_os = "emscripten")]
mod emscripten {
    use std::ffi::c_void;
    #[allow(non_camel_case_types)]
    type c_int = i32;
    #[allow(non_camel_case_types)]
    pub type EM_BOOL = c_int;

    extern "C" {
        pub fn emscripten_request_animation_frame_loop(
            func: extern "C" fn(millis: f64, user_data: *mut c_void) -> EM_BOOL,
            arg: *mut c_void,
        );

        pub fn emscripten_throw_string(utf8_string: *const u8);
    }
}

#[cfg(not(target_os = "emscripten"))]
mod emscripten {
    use std::ffi::c_void;
    #[allow(non_camel_case_types)]
    type c_int = i32;
    #[allow(non_camel_case_types)]
    pub type EM_BOOL = c_int;

    pub unsafe fn emscripten_request_animation_frame_loop(
        _func: extern "C" fn(millis: f64, user_data: *mut c_void) -> EM_BOOL,
        _arg: *mut c_void,
    ) {
        unreachable!("Should only be called for emscripten targets")
    }

    pub unsafe fn emscripten_throw_string(_utf8_string: *const u8) {
        unreachable!("Should only be called for emscripten targets")
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn init_log() {
    env_logger::init();
}

#[cfg(target_arch = "wasm32")]
fn init_log() {
    let _ = if cfg!(debug_assertions) {
        console_log::init()
    } else {
        console_log::init_with_level(log::Level::Warn)
    };
}

fn main() {
    init_log();

    let sdl_context = sdl2::init().unwrap();

    let video_subsystem = sdl_context.video().unwrap();
    let mut window = Window::create(video_subsystem);
    let mut video = Video::setup(&mut window);
    let mut events = sdl_context.event_pump().unwrap();

    let mut emulator = tihle::Emulator::new();
    let mut cpu = tihle::Z80::new();

    if let Some(path) = std::env::args().skip(1).next() {
        load_program(&mut emulator, &mut cpu, &path);
    }

    if cfg!(target_os = "emscripten") {
        type State<'a> = (Option<f64>, Video<'a>, sdl2::EventPump, Emulator, Z80);
        let mut state: State<'_> = (None, video, events, emulator, cpu);

        extern "C" fn wrap_iterate(millis: f64, user_data: *mut c_void) -> emscripten::EM_BOOL {
            // Simulating an infinite loop effectively leaks the current stack, promoting
            // any live stack allocations to the static lifetime.
            let state: &mut State<'static> = unsafe { &mut *(user_data as *mut State) };
            let frame_time = match state.0 {
                Some(prev_millis) => Duration::from_secs_f64((millis - prev_millis) / 1000.0),
                ref mut x @ None => {
                    *x = Some(millis);
                    return 1;
                }
            };

            iterate_main(
                frame_time,
                &mut state.1,
                &mut state.2,
                &mut state.3,
                &mut state.4,
            );
            1
        }
        unsafe {
            emscripten::emscripten_request_animation_frame_loop(
                wrap_iterate,
                &mut state as *mut State as *mut _,
            );
            // Same behavior as emscripten_set_main_loop(simulate_infinite_loop=true): break out
            // back into the browser event loop never to return, but don't unwind the stack
            // so locals remain live.
            emscripten::emscripten_throw_string(b"unwind\0".as_ptr());
        }
        unreachable!();
    }

    let target_frame_time = Duration::from_secs(1) / 60;
    loop {
        let frame_start = Instant::now();
        if iterate_main(
            target_frame_time,
            &mut video,
            &mut events,
            &mut emulator,
            &mut cpu,
        ) {
            break;
        }

        let elapsed = frame_start.elapsed();
        // TODO turning on vsync will probably ensure accurate framerate by blocking
        // until end of frame more precisely, and matches the emscripten animation behavior
        // where we don't know the framerate.
        match target_frame_time.checked_sub(frame_start.elapsed()) {
            None => {
                warn!(
                    "Running slowly: emulating {:?} took {:?}",
                    target_frame_time, elapsed
                );
            }
            Some(wait) => {
                std::thread::sleep(wait);
            }
        }
    }
}

fn load_program(emulator: &mut Emulator, mut cpu: &mut Z80, path: &str) {
    match File::open(path) {
        Ok(f) => {
            if let Err(e) = emulator.load_program(&mut cpu, f) {
                error!("Failed to load program from {}: {:?}", path, e);
            }
        }
        Err(e) => {
            error!("Unable to open {} to load: {}", path, e);
        }
    }
}

/// Run a single iteration of emulation, until the emulated CPU has run for `frame_time`.
///
/// Returns true if the program should exit.
fn iterate_main(
    mut frame_time: Duration,
    video: &mut Video,
    events: &mut sdl2::EventPump,
    emu: &mut Emulator,
    cpu: &mut Z80,
) -> bool {
    // Process events
    for event in events.poll_iter() {
        use sdl2::event::Event;

        match event {
            Event::KeyDown {
                keycode: Some(k), ..
            } => {
                if let Some(k) = translate_keycode(k) {
                    debug!("Key down: {:?}", k);
                    emu.keyboard.key_down(k);
                } else {
                    debug!("Ignoring unhandled key {:?}", k);
                }
            }
            Event::KeyUp {
                keycode: Some(k), ..
            } => {
                if let Some(k) = translate_keycode(k) {
                    debug!("Key up: {:?}", k);
                    emu.keyboard.key_up(k);
                }
            }
            Event::DropFile { filename, .. } => {
                emu.reset();
                match File::open(filename) {
                    Ok(f) => {
                        if let Err(e) = emu.load_program(cpu, f) {
                            error!("Failed to load program: {:?}", e);
                        }
                    }
                    Err(e) => {
                        error!("Unable to open file to load: {}", e);
                    }
                }
            }
            Event::Quit { .. } => {
                return true;
            }
            _ => {}
        }
    }

    if emu.is_running() {
        // Loop running the CPU to reach the target emulated time
        const ZERO_TIME: Duration = Duration::from_nanos(0);
        while frame_time != ZERO_TIME {
            debug!("Run CPU for up to {:?}", frame_time);
            let emulated_duration = emu.run(cpu, frame_time);
            debug!("CPU ran for {:?}", emulated_duration);
            frame_time = frame_time
                .checked_sub(emulated_duration)
                .unwrap_or(ZERO_TIME);
        }
    }

    debug!("CPU run complete; swap display");
    video.update(&emu.display);
    false
}

fn translate_keycode(keycode: sdl2::keyboard::Keycode) -> Option<tihle::keyboard::Key> {
    use sdl2::keyboard::Keycode;
    use tihle::keyboard::Key;

    Some(match keycode {
        Keycode::Left => Key::Left,
        Keycode::Up => Key::Up,
        Keycode::Right => Key::Right,
        Keycode::Down => Key::Down,
        Keycode::LCtrl | Keycode::RCtrl => Key::Second,
        Keycode::LShift | Keycode::RShift => Key::Alpha,
        Keycode::Num0 | Keycode::Kp0 => Key::Zero,
        Keycode::Num1 | Keycode::Kp1 => Key::One,
        Keycode::Num2 | Keycode::Kp2 => Key::Two,
        Keycode::Num3 | Keycode::Kp3 => Key::Three,
        Keycode::Num4 | Keycode::Kp4 => Key::Four,
        Keycode::Num5 | Keycode::Kp5 => Key::Five,
        Keycode::Num6 | Keycode::Kp6 => Key::Six,
        Keycode::Num7 | Keycode::Kp7 => Key::Seven,
        Keycode::Num8 | Keycode::Kp8 => Key::Eight,
        Keycode::Num9 | Keycode::Kp9 => Key::Nine,
        Keycode::Plus | Keycode::KpPlus => Key::Plus,
        Keycode::Minus | Keycode::KpMinus => Key::Minus,
        Keycode::KpMultiply => Key::Multiply,
        Keycode::KpDivide | Keycode::Slash => Key::Divide,
        Keycode::Return | Keycode::KpEnter => Key::Enter,
        Keycode::Period | Keycode::KpPeriod => Key::Period,
        Keycode::Backspace => Key::Clear,
        _ => return None,
    })
}
