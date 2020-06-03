#[macro_use]
extern crate log;

#[macro_use]
extern crate num_derive;

use std::cell::Cell;
use std::time::{Duration, Instant};

/*

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

*/

mod bcalls;
mod checksum;
mod display;
mod interrupt;
mod memory;
mod tifiles;
mod z80;

pub use interrupt::InterruptController;
pub use memory::Memory;
use std::ops::RangeInclusive;
pub use z80::Z80;

pub struct Emulator {
    clock_rate: u32,
    pub mem: Memory,
    pub interrupt_controller: InterruptController,
    pub display: display::Display,

    target_framerate: u32,
    /// If true, emulation has terminated.
    terminate: Cell<bool>,
}

impl Emulator {
    pub fn new() -> Self {
        Emulator {
            clock_rate: 6_000_000,
            mem: Memory::new(),
            interrupt_controller: InterruptController::new(),
            display: display::Display::new(),

            target_framerate: 60,
            terminate: Cell::new(false),
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Cycle count to force the core to stop executing.
    ///
    /// This is an arbitrarily large number of cycles, but not so large that
    ///
    const FORCE_YIELD: usize = usize::MAX / 2;

    fn duration_to_cycles(&self, duration: Duration) -> usize {
        let cycle_secs = 1.0 / self.clock_rate as f64;

        (duration.as_secs_f64() / cycle_secs) as usize
    }

    pub fn run(&mut self, cpu: &mut Z80, frame_start: &Instant) {
        let frame_duration = Duration::from_nanos(1e9 as u64 / self.target_framerate as u64);

        if self.terminate.get() {
            debug!("CPU terminated, sleeping for a frame");
            std::thread::sleep(frame_duration);
            return;
        }

        let (irq_pending, until_next_interrupt) = self.interrupt_controller.poll();
        trace!(
            "IRQ pending: {}; next interrupt: {:?}",
            irq_pending,
            until_next_interrupt
        );
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

        if cpu.is_halted() || self.terminate.get() {
            trace!("CPU halted, sleep {:?}", step_duration);
            std::thread::sleep(step_duration);
        } else {
            trace!(
                "Run CPU for {:?} ({} cycles)",
                step_duration,
                self.duration_to_cycles(step_duration)
            );
            cpu.run(self.duration_to_cycles(step_duration), self);

            // The CPU ran, and probably ran faster than real time; wait until
            // wall time catches up.
            let step_elapsed = frame_start.elapsed();
            if let Some(t) = step_duration.checked_sub(step_elapsed) {
                trace!("CPU {:?} ahead; sleeping to catch up", t);
                std::thread::sleep(t);
            } else {
                warn!(
                    "Running slowly: emulating {}ms took {}ms on the wall",
                    step_duration.as_millis(),
                    step_elapsed.as_millis()
                );
            }
        }
    }

    pub fn load_program<R: std::io::Read>(
        &mut self,
        cpu: &mut Z80,
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

        let regs = cpu.regs();

        let code_size = internal_len - 2;
        let load_addr = 0x9d95u16; // userMem

        debug!("Loading {} byte(s) of code to {:04X}", code_size, load_addr);
        self.mem[load_addr..load_addr + code_size].copy_from_slice(&var.data[4..]);

        // Set up stack to return to the reset vector at exit.
        self.mem[0xfffe] = 0;
        self.mem[0xffff] = 0;
        regs.sp = 0xfffe;
        // Begin executing at load address
        regs.pc = load_addr as u16;

        // Enable interrupts in mode 1
        regs.set_interrupt_enable(true);
        regs.set_im(1);

        Ok(var)
    }

    const RAM_ADDRS: RangeInclusive<u16> = 0x8000..=0xFFFF;

    #[inline]
    fn read_memory(&mut self, core: &mut Z80, addr: u16) -> u8 {
        trace!("Memory read from {:04X}", addr);
        if Self::RAM_ADDRS.contains(&addr) {
            return self.mem[addr];
        }

        let mut elapsed: usize = 0;
        let read_byte: u8 = match addr {
            0x0000 => {
                info!("Trapped reset; terminating CPU.");
                self.terminate.set(true);
                elapsed = Self::FORCE_YIELD;
                0xc7 // rst 00h; infinite loop
            }

            0x0028 => {
                elapsed = bcalls::bcall_trap(self, core);
                0xc9 // return normally after trap
            }

            0x0038 => {
                0xed // reti byte 1
            }
            0x0039 => {
                0x4d // reti byte 2
            }

            _ => {
                error!(
                    "Unhandled memory read from {:#06x}! {:#?}",
                    addr,
                    core.regs()
                );
                0xc9 // ret (try to limp along)
            }
        };

        core.z80.cycles += elapsed;
        read_byte
    }

    #[inline]
    fn write_memory(&mut self, _core: &mut Z80, addr: u16, value: u8) {
        if Self::RAM_ADDRS.contains(&addr) {
            self.mem[addr] = value;
        } else {
            warn!("Unhandled memory write to {:#06x}", addr);
        }
    }

    fn wait_for_interrupt(&mut self, _core: &mut Z80) {
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
