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
extern crate arr_macro;

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
use std::time::Duration;

mod bcalls;
mod checksum;
pub mod debug;
pub mod display;
mod interrupt;
pub mod keyboard;
pub mod memory;
mod tifiles;
mod traps;
pub mod z80;

pub mod include {
    pub mod ion;
    pub mod mirageos;
    pub mod tios;
}

pub use display::Display;
pub use interrupt::InterruptController;
pub use memory::Memory;
pub use z80::{Flags, Z80};

pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

pub struct Emulator {
    clock_rate: u32,
    pub mem: Memory,
    pub interrupt_controller: InterruptController,
    pub display: Display,
    pub keyboard: keyboard::Keyboard,
    /// If true, emulation has terminated.
    terminate: Cell<bool>,
    #[cfg(feature = "remote-debug")]
    pub debug: debug::RemoteDebugger,
    #[cfg(not(feature = "remote-debug"))]
    debug: debug::DummyDebugger,

    #[cfg(test)]
    pub debug_commands_executed: usize,
}

pub struct Builder {
    flash_pages: [Vec<u8>; 0x20],
}

static DEFFAULT_FLASH_IMAGE: &[(u8, &[u8])] = &[
    (0, include_bytes!("../os/page00.bin")),
    (1, include_bytes!("../os/page01.bin")),
    (0x1B, include_bytes!("../os/page1b.bin")),
    (4, include_bytes!("mirageos.bin")),
];

/// Kinds of memory access
#[derive(Debug, PartialEq, Eq)]
enum MemoryAccessKind {
    /// The data being accessed is treated as an instruction
    Instruction,
    /// The data being accessed is treated as plain data
    Data,
}

impl std::default::Default for Builder {
    fn default() -> Self {
        let mut flash_pages = arr![vec![]; 0x20];
        for (page, data) in DEFFAULT_FLASH_IMAGE {
            flash_pages[*page as usize].extend(data.iter());
        }

        Builder { flash_pages }
    }
}

impl Builder {
    pub fn build(self) -> Emulator {
        Emulator {
            clock_rate: 6_000_000,
            mem: Memory::new((0u8..).zip(self.flash_pages.iter())),
            interrupt_controller: InterruptController::new(),
            display: Display::new(),
            keyboard: keyboard::Keyboard::new(),
            terminate: Cell::new(true),
            debug: Default::default(),

            /// For tests, report the number of debug commands that have been executed.
            ///
            /// This allows tests to wait until their commands have been processed, since
            /// the time it takes to handle them may vary due to thread scheduling.
            #[cfg(test)]
            debug_commands_executed: 0,
        }
    }
}

impl Emulator {
    /// Construct a new emulator.
    ///
    /// Initially the CPU is terminated; call [load_program] to start the
    /// CPU so calls to [run] will run the CPU.
    pub fn new() -> Self {
        Self::builder().build()
    }

    pub fn builder() -> Builder {
        Default::default()
    }

    pub fn reset(&mut self) {
        self.interrupt_controller = InterruptController::new();
        self.display = Display::new();
        self.keyboard = keyboard::Keyboard::new();
        self.terminate = Cell::new(true);
    }

    pub fn is_running(&self) -> bool {
        !self.terminate.get()
    }

    fn duration_to_cycles(&self, duration: Duration) -> usize {
        let cycle_secs = 1.0 / self.clock_rate as f64;

        // Running for very short intervals might be less than a cycle; always run
        // at least 1 cycle.
        (duration.as_secs_f64() / cycle_secs).ceil() as usize
    }

    fn cycles_to_duration(&self, cycles: usize) -> Duration {
        Duration::from_secs_f64(cycles as f64 / self.clock_rate as f64)
    }

    /// Run the emulator for up to `max_step`, returning the amount of time
    /// the emulated CPU ran for or None if the CPU is not running.
    pub fn run(&mut self, cpu: &mut Z80, max_step: Duration) -> Option<Duration> {
        // Always process debugger actions.
        let _actions = self.debug.run();
        #[cfg(test)]
        {
            self.debug_commands_executed += _actions;
        }

        if !self.is_running() {
            debug!("CPU terminated, doing nothing");
            return None;
        }

        let (irq_pending, until_next_interrupt) = self.interrupt_controller.poll();
        debug!(
            "IRQ pending: {}; next interrupt: {:?}",
            irq_pending, until_next_interrupt
        );
        cpu.set_irq(irq_pending);

        // Run the CPU for the requested time or until the next interrupt,
        // whichever is sooner.
        // TODO: this won't handle enabling interrupts while running;
        // the core needs to be able to break based on requests from
        // callbacks to do this accurately.
        let step_duration = match until_next_interrupt {
            None => max_step,
            Some(t) => std::cmp::min(max_step, t),
        };

        let duration_run = if cpu.is_halted() && !irq_pending {
            debug!("CPU halted, wait {:?} for interrupt", step_duration);
            step_duration
        } else {
            debug!(
                "Run CPU for {:?} ({} cycles)",
                step_duration,
                self.duration_to_cycles(step_duration)
            );
            let cycles_run = cpu.run(self.duration_to_cycles(step_duration), self);
            self.cycles_to_duration(cycles_run)
        };

        self.interrupt_controller.advance(duration_run);
        Some(duration_run)
    }

    /// Load an 8xp-format program from the given reader.
    ///
    /// The program will be loaded at 9D95 with the CPU set to begin execution there,
    /// and the system state will be set up consistent with the detected type of program
    /// (such as setting up the context for the appropriate shell). The emulator
    /// will be reset so [run] will run the CPU.
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

        self.terminate.set(false);
        Ok(var)
    }

    fn setup_tios_context(&mut self, core: &mut Z80) {
        let regs = core.regs_mut();
        use include::tios;

        // Enable interrupts in mode 1
        regs.set_interrupt_enable(true);
        regs.set_im(1);

        // IY points to flags
        regs.iy = tios::flags;

        // All flags are reset
        for addr in tios::flags..tios::flags + 0x46 {
            self.mem[addr] = 0;
        }

        // The VAT is empty
        self.mem.write_u16(tios::progPtr, tios::symTable);
        self.mem.write_u16(tios::pTemp, tios::symTable);

        // The unused hardware stack area is zeroed
        for addr in tios::symTable + 1..regs.sp {
            self.mem[addr] = 0;
        }

        // Cursor and pen are at top left
        for &byte in &[tios::curRow, tios::curCol, tios::penRow, tios::penCol] {
            self.mem[byte] = 0;
        }
    }

    #[inline]
    fn read_memory(&mut self, _core: &mut Z80, addr: u16, access_kind: MemoryAccessKind) -> u8 {
        let byte = self.mem[addr];
        trace!("Memory read {:?} {:04X} -> {:02X}", access_kind, addr, byte);
        byte
    }

    #[inline]
    fn write_memory(&mut self, core: &mut Z80, addr: u16, value: u8) {
        trace!("Memory write {:02X} -> {:04X}", value, addr);
        if self.mem.put(addr, value).is_err() {
            info!("{:#?}", core.regs());
        }
    }

    fn write_io(&mut self, cpu: &mut Z80, port: u8, value: u8) {
        match port {
            0x01 => self.keyboard.set_active_mask(value),
            0x03 => {
                self.interrupt_controller.write_mask_port(value);
                // TODO since this can affect scheduling, we should actually force
                // the CPU to yield so we can run until the next interrupt that may
                // be sooner than originally thought.
                let (pending, _) = self.interrupt_controller.poll();
                debug!("Port 3 write {:02X} sets IRQ={}", value, pending);
                cpu.set_irq(pending);
            }
            0x06 => self.mem.set_bank_a_page(value),
            0x10 => self.display.write_control(value),
            0x11 => self.display.write_data(value),
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
            0x01 => self.keyboard.read(),
            0x03 => self.interrupt_controller.read_mask_port(),
            0x04 => self.interrupt_controller.read_status_port(),
            0x06 => self.mem.get_bank_a_page(),
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
