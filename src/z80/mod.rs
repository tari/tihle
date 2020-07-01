use crate::Emulator;
use bitflags::bitflags;
use std::ffi::c_void;
use std::ptr;

mod ffi;

pub struct Z80 {
    pub z80: ffi::Z80,
}

bitflags! {
    pub struct Flags: u8 {
        const S = 0x80;
        const Z = 0x40;
        const H = 0x10;
        const PV = 0x04;
        const N = 0x02;
        const C = 0x01;
    }
}

type FfiContext = (*mut Z80, *mut Emulator);

impl Z80 {
    pub fn new() -> Self {
        let mut out = Z80 {
            z80: ffi::Z80 {
                cycles: 0,
                context: ptr::null_mut(),
                read: Some(Self::handle_read),
                write: Some(Self::handle_write),
                port_in: Some(Self::handle_io_read),
                port_out: Some(Self::handle_io_write),
                int_data: Some(Self::handle_mode0_vector),
                halt: None,
                trap: Some(Self::handle_trap),
                ..ffi::Z80::new()
            },
        };

        unsafe {
            ffi::z80_power(&mut out.z80 as *mut _, 1);
        }

        out
    }

    unsafe fn ctx_from_ptr<'a>(p: *mut c_void) -> (&'a mut Z80, &'a mut Emulator) {
        let (core, ctx) = *(p as *mut FfiContext as *const FfiContext);

        (&mut *core, &mut *ctx)
    }

    extern "C" fn handle_read(ctx: *mut c_void, address: u16) -> u8 {
        let (core, emu) = unsafe { Self::ctx_from_ptr(ctx) };

        emu.read_memory(core, address)
    }

    extern "C" fn handle_write(ctx: ffi::Ctx, address: u16, value: u8) {
        let (core, emu) = unsafe { Self::ctx_from_ptr(ctx) };

        emu.write_memory(core, address, value)
    }

    extern "C" fn handle_io_read(ctx: ffi::Ctx, address: u16) -> u8 {
        let (core, emu) = unsafe { Self::ctx_from_ptr(ctx) };

        emu.read_io(core, address as u8)
    }

    extern "C" fn handle_io_write(ctx: ffi::Ctx, address: u16, value: u8) {
        let (core, emu) = unsafe { Self::ctx_from_ptr(ctx) };

        emu.write_io(core, address as u8, value);
    }

    extern "C" fn handle_trap(ctx: ffi::Ctx, trap_no: u16) -> usize {
        let (core, emu) = unsafe { Self::ctx_from_ptr(ctx) };
        emu.trap(trap_no, core)
    }

    #[cold]
    extern "C" fn handle_mode0_vector(_ctx: ffi::Ctx) -> u32 {
        0
    }

    /// Run the core for the given number of cycles.
    ///
    /// The number of cycles to run for is inexact, because most instruction
    /// take more than one cycle to run; this function will return only on
    /// instruction boundaries.
    ///
    /// The provided `Ctx` is passed to traps for access to higher-level
    /// system state.
    pub fn run(&mut self, cycles: usize, ctx: &mut Emulator) -> usize {
        assert!(
            cycles > 0,
            "Running the CPU for zero cycles doesn't make sense"
        );
        // Safe: we pass mutable refs down into the core, which effecitvely
        // passes them back to callbacks by copying the parameter it receives;
        // there is no aliasing because it's just copying refs it has down to
        // a leaf function.
        let ffi_ctx: FfiContext = (self as *mut Self, ctx as *mut Emulator);
        self.z80.context = &ffi_ctx as *const _ as *mut c_void;
        unsafe { ffi::z80_run(&mut self.z80 as *mut _, cycles) }
    }

    pub fn regs_mut(&mut self) -> &mut ffi::State {
        &mut self.z80.regs
    }

    pub fn regs(&self) -> &ffi::State {
        &self.z80.regs
    }

    pub fn flags(&self) -> Flags {
        unsafe { Flags::from_bits_unchecked(self.regs().af as u8) }
    }

    pub fn set_flags(&mut self, flags: Flags) {
        let regs = self.regs_mut();
        regs.af &= 0xFF00;
        regs.af |= flags.bits() as u16;
    }

    pub fn set_irq(&mut self, pending: bool) {
        unsafe { ffi::z80_int(&mut self.z80 as *mut _, pending as u8) }
    }

    pub fn is_halted(&self) -> bool {
        self.z80.is_halted()
    }
}
