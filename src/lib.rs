//! A TI-8x calculator emulator.
//!
//! While the 8x series (83+/SE, 84+/SE) are simple to emulate and there are numerous other
//! emulators out there, tihle is unique in that it is meant to run programs without depending
//! on any nonfree code.
//!
//! ## Traps
//!
//! Traps make emulation feasible without depending on a complete software implementation by
//! allowing chosen code to trap into emulator-provided code that is often simpler to implement
//! and more performant than equivalent emulated code would be.
//!
//! The CPU traps on executing the instruction `ED 25 nn nn`. This is unusual as it is a 4-byte
//! instruction unlike any others on the Z80, but it is not known to be a useful undocumented
//! instruction on the Z80, nor is it defined on the eZ80.
//!
//! The 16-bit value (`nn nn`) in the trap instruction identifies the action to be taken by the
//! emulator in response to the trap.

#[macro_use]
extern crate log;

#[macro_use]
extern crate num_derive;

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use]
extern crate quickcheck_macros;

use num_traits::FromPrimitive;
use std::cell::Cell;
use std::time::{Duration, Instant};

/*

Obviously required traps:
 * 0028: bcall entry point
 * 0038: IM 1 vector

Likely required traps:
 * The other documented rst vectors
   * 08 => OP1ToOP2
   * 10 => FindSym
   * 18 => PushRealO1
   * 20 => Mov9ToOP1
   * 30 => FPAdd
 * LCD_BUSY_QUICK (0x000B; 47 cycles)
 * Ion vectors
 * MOS vectors

Major bcalls:
 * PutS and friends (honor flags too)
 * ChkFindSym

Ports that we probably need good fidelity for:
 * Keypad ports
 * LCD ports

Used by phoenix:
 * _Disphl
 * _VPutMap
 * _ClrLCDFull
 * idetect
 * _divhlby10
 * _VPutS
 * _GetCSC
 * flags + 13 (appFlags)
 * LCD ports
 * Mirage setupint

*/

mod bcalls;
mod checksum;
pub mod display;
mod interrupt;
pub mod memory;
mod tifiles;
mod traps;
pub mod z80;

pub mod include {
    pub mod ion;
    pub mod mirageos;
    pub mod tios;
}

pub use interrupt::InterruptController;
pub use memory::Memory;
pub use z80::{Flags, Z80};

pub struct Emulator {
    clock_rate: u32,
    pub mem: Memory,
    pub interrupt_controller: InterruptController,
    pub display: display::Display,

    target_framerate: u32,
    /// If true, emulation has terminated.
    terminate: Cell<bool>,
}

static FLASH_IMAGE: &[(u8, &[u8])] = &[
    (0, include_bytes!("../os/page00.bin")),
    (1, include_bytes!("../os/page01.bin")),
    (0x1B, include_bytes!("../os/page1b.bin")),
    (4, include_bytes!("mirageos.bin")),
];

impl Emulator {
    pub fn new() -> Self {
        Emulator {
            clock_rate: 6_000_000,
            mem: Memory::new(FLASH_IMAGE),
            interrupt_controller: InterruptController::new(),
            display: display::Display::new(),

            target_framerate: 60,
            terminate: Cell::new(false),
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn is_running(&self) -> bool {
        !self.terminate.get()
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

        if !self.is_running() {
            debug!("CPU terminated, sleeping for a frame");
            std::thread::sleep(frame_duration);
            return;
        }

        let (irq_pending, until_next_interrupt) = self.interrupt_controller.poll();
        debug!(
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
        let mut var = file.var;
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
        let uses_ion_libraries = var.patch_ion_program();
        var.patch_mos_program();

        let code_size = internal_len - 2;
        let load_addr = 0x9d95u16; // userMem
        debug!("Loading {} byte(s) of code to {:04X}", code_size, load_addr);
        self.mem[load_addr..load_addr + code_size].copy_from_slice(&var.data[4..]);

        let regs = cpu.regs_mut();
        // Set up stack to return to the reset vector at exit.
        self.mem[0xfffe] = 0;
        self.mem[0xffff] = 0;
        regs.sp = 0xfffe;
        // Begin executing at load address
        regs.pc = load_addr as u16;

        if uses_ion_libraries {
            use include::{ion, mirageos};
            // Set up the Ion vector table, aliased to the Mirage vectors.
            let mut vector = |addr: u16, target: u16| {
                self.mem[addr] = 0xc3; // unconditional jp
                self.mem.write_u16(addr + 1, target);
            };
            vector(ion::ionVersion, mirageos::ionVersion);
            vector(ion::ionRandom, mirageos::ionRandom);
            vector(ion::ionPutSprite, mirageos::ionPutSprite);
            vector(ion::ionLargeSprite, mirageos::ionLargeSprite);
            vector(ion::ionGetPixel, mirageos::ionGetPixel);
            vector(ion::ionFastCopy, mirageos::ionFastCopy);
            vector(ion::ionDetect, mirageos::ionDetect);
            vector(ion::ionDecompress, mirageos::ionDecompress);
        }

        self.setup_tios_context(cpu);
        // Map Mirage into bank A
        self.mem.set_bank_a_page(4);

        Ok(var)
    }

    fn setup_tios_context(&mut self, core: &mut Z80) {
        let regs = core.regs_mut();

        // Enable interrupts in mode 1
        regs.set_interrupt_enable(true);
        regs.set_im(1);

        // IY points to flags
        regs.iy = include::tios::flags;

        // TODO we may need to set up the VAT and other things for Mirage.
    }

    #[inline]
    fn read_memory(&mut self, _core: &mut Z80, addr: u16) -> u8 {
        let byte = self.mem[addr];
        trace!("Memory read {:04X} -> {:02X}", addr, byte);
        byte
    }

    #[inline]
    fn write_memory(&mut self, core: &mut Z80, addr: u16, value: u8) {
        trace!("Memory write {:02X} -> {:04X}", value, addr);
        if self.mem.put(addr, value).is_err() {
            info!("{:#?}", core.regs());
        }
    }

    fn wait_for_interrupt(&mut self, _core: &mut Z80) {
        unimplemented!()
    }

    fn write_io(&mut self, _core: &mut Z80, port: u8, value: u8) {
        match port {
            0x10 => {
                self.display.write_control(value);
            }
            0x11 => {
                self.display.write_data(value);
            }
            0xFF if self.is_running() => {
                info!("Got write to port 255; terminating emulation");
                self.terminate.set(true);
            }
            _ => {
                warn!(
                    "Unhandled port write to {:#04x} (value={:#04x})",
                    port, value
                );
            }
        }
    }

    fn read_io(&mut self, _core: &mut Z80, port: u8) -> u8 {
        match port {
            0x10 => self.display.read_status(),
            0x11 => self.display.read_data(),
            _ => {
                warn!("Unhandled port read from {:#04x}", port);
                0
            }
        }
    }

    fn trap(&mut self, trap_no: u16, core: &mut Z80) -> usize {
        if let Some(trap) = traps::Trap::from_u16(trap_no) {
            trap.handle(self, core)
        } else {
            panic!("Unrecognized trap: {:04X}", trap_no);
        }
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
