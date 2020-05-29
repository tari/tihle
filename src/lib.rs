#[macro_use]
extern crate log;

#[macro_use]
extern crate num_derive;

use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{Duration, Instant};

/*

Setup:
 * Generate dummy memory map; stack, VAT..?
 * Reset CPU
 * Set IM 1

Loading programs requires checking the type bytes, only support tasmCmp for now.

# Traps

Reading from an address that is defined as a trap returns an instruction
sequence appropriate to the trap, and on the final instruction the trap
executes over the CPU state.

In most cases simply "ret" is the appropriate return sequence for a trap,
but some may want a different sequence. Interrupt vectors for instance
probably want "reti".

Each trap consumes an arbitrary number of cycles, to simulate the time it
takes to execute on hardware. In particular LCD copy traps are very slow.

Obviously required traps:
 * 0028: bcall entry point
 * 0038: IM 1 vector

Likely required traps:
 * The other documented rst vectors
 * Ion vectors
 * MOS vectors

Major bcalls:
 * PutS and friends (honor flags too)
 * ChkFindSym

Ports that we probably need good fidelity for:
 * Keypad ports
 * LCD ports

Questions:
 * How often does the calculator fire interrupts? We need to do that
   ourselves.

*/

mod bcalls;
mod checksum;
pub mod display;
mod interrupt;
mod memory;
pub mod tifiles;
pub mod z80;

pub use interrupt::InterruptController;
pub use memory::Memory;
use std::borrow::BorrowMut;
pub use z80::Z80;

type Trap = dyn FnMut(&mut Emulator) -> (u8, usize);

pub struct Emulator {
    cpu: Z80,
    clock_rate: u32,
    pub mem: Memory,
    pub interrupt_controller: InterruptController,
    traps: HashMap<u16, Box<Trap>>,

    events: sdl2::EventPump,
    target_framerate: u32,
    display: display::Display<sdl2::video::Window>,
    /// If true, emulation has terminated.
    terminate: Rc<Cell<bool>>,
}

impl Emulator {
    pub fn new() -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video = sdl_context.video().unwrap();

        let canvas = video
            .window("tihle", 96, 64)
            .build()
            .unwrap()
            .into_canvas()
            .build()
            .unwrap();

        let terminate_flag = Rc::new(Cell::new(false));

        // TODO use a halt handler to sleep and whatever
        let mut cpu = Z80::new();
        let mut mem = Memory::new();
        let mut traps = HashMap::new();
        /*
        let reset_terminate = terminate_flag.clone();
        cpu.add_trap(0x0000, move |_core: z80::CoreState| {
            reset_terminate.set(true);
            // Arbitrarily long amount of time to make the core yield, not
            // trusting it to work correctly with usize::MAX.
            clock_rate as usize
        });
        cpu.add_trap(0x0028, bcalls::bcall_trap);
         */

        Emulator {
            cpu,
            clock_rate: 6_000_000,
            mem,
            traps,
            interrupt_controller: InterruptController::new(),

            events: sdl_context.event_pump().unwrap(),
            target_framerate: 60,
            display: display::Display::new(canvas),
            terminate: terminate_flag.clone(),
        }
    }

    fn duration_to_cycles(&self, duration: Duration) -> usize {
        let cycle_secs = 1.0 / self.clock_rate as f64;

        (duration.as_secs_f64() / cycle_secs) as usize
    }

    pub fn run(&mut self, cpu: &mut Z80) {
        let frame_duration = Duration::from_nanos(1e9 as u64 / self.target_framerate as u64);

        loop {
            let frame_start = Instant::now();

            // Process events
            for event in self.events.poll_iter() {
                use sdl2::event::Event;

                if let Event::KeyDown {
                    keycode: Some(k), ..
                } = event
                {
                    println!("key down: {}", k);
                }
            }

            if self.interrupt_controller.is_pending() {
                self.cpu.set_irq(true);
            }

            let (irq_pending, until_next_interrupt) = self.interrupt_controller.poll();
            cpu.set_irq(irq_pending);

            // Run the CPU for a frame or until the next interrupt,
            // whichever is sooner.
            // TODO: this won't handle enabling interrupts while running;
            // the core needs to be able to break based on requests from
            // callbacks to do this accurately. Seems like it needs a few
            // extra hooks, like one for fetching instructions mostly that
            // can say "pretend this took a while" or "stop running so the
            // caller can do some work for us."
            let step_duration = match until_next_interrupt {
                None => frame_duration,
                Some(t) => std::cmp::min(frame_duration, t),
            };

            if cpu.is_halted() {
                std::thread::sleep(step_duration);
            } else {
                cpu.run(self.duration_to_cycles(step_duration), self);
                if self.terminate.get() {
                    break;
                }

                // The CPU ran, and probably ran faster than real time; wait until
                // wall time catches up.
                let step_elapsed = frame_start.elapsed();
                if let Some(t) = step_duration.checked_sub(step_elapsed) {
                    std::thread::sleep(t);
                } else {
                    warn!("Running slowly: emulating {}ms took {}ms on the wall",
                          step_duration.as_millis(),
                          step_elapsed.as_millis());
                }
            }


        }
    }

    pub fn load_program<R: std::io::Read>(
        &mut self,
        r: R,
    ) -> Result<tifiles::Variable, LoadProgramError> {
        use tifiles::{File, VariableType};

        let file = File::read_from(r)?;
        let var = file.var;
        if var.ty != VariableType::Program && var.ty != VariableType::ProtectedProgram {
            return Err(LoadProgramError::UnsupportedType);
        }

        let internal_len = var.data[0] as u16 | (var.data[1] as u16) << 8;
        if internal_len != (var.data.len() - 2) as u16 {
            return Err(LoadProgramError::IncorrectLength);
        }

        // t2ByteTok, tAsmCmp signature marks an assembly program
        if var.data[2..4] != b"\xbb\x6d"[..] {
            return Err(LoadProgramError::InvalidSignature);
        }

        let regs = self.cpu.regs();

        let code_size = internal_len - 2;
        let load_addr = 0x9d95u16; // userMem

        self.mem[load_addr..load_addr + code_size].copy_from_slice(&var.data[4..]);

        // Set up stack to return to the reset vector at exit.
        self.mem[0xfffe] = 0;
        self.mem[0xffff] = 0;
        regs.sp = 0xfffe;
        // Begin executing at load address
        regs.pc = load_addr as u16;

        Ok(var)
    }

    fn read_memory(&mut self, core: &mut Z80, addr: u16) -> u8 {
        unimplemented!()
    }

    fn write_memory(&mut self, core: &mut Z80, addr: u16, value: u8) {
        unimplemented!()
    }

    fn wait_for_interrupt(&mut self, core: &mut Z80) {
        unimplemented!()
    }
}

#[derive(Debug)]
pub enum LoadProgramError {
    FileRead(tifiles::Error),
    Io(std::io::Error),
    UnsupportedType,
    InvalidSignature,
    /// The internal length field does not match the actual size.
    IncorrectLength,
}

impl std::convert::From<std::io::Error> for LoadProgramError {
    fn from(other: std::io::Error) -> Self {
        LoadProgramError::Io(other)
    }
}

impl std::convert::From<tifiles::Error> for LoadProgramError {
    fn from(other: tifiles::Error) -> Self {
        LoadProgramError::FileRead(other)
    }
}
