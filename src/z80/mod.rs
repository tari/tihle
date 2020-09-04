use crate::debug::Debugger;
use crate::Emulator;
use bitflags::bitflags;
use std::ffi::c_void;
use std::ops::DerefMut;
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

type FfiContext = (*mut Z80, *mut Emulator, *mut Debugger);

impl Z80 {
    pub fn new() -> Self {
        let mut out = Z80 {
            z80: ffi::Z80 {
                cycles: 0,
                context: ptr::null_mut(),
                ..ffi::Z80::new()
            },
        };

        unsafe {
            ffi::z80_power(&mut out.z80 as *mut _, 1);
        }

        out
    }

    unsafe fn ctx_from_ptr<'a>(
        p: *mut c_void,
    ) -> (&'a mut Z80, &'a mut Emulator, Option<&'a mut Debugger>) {
        let (core, ctx, dbg) = *(p as *mut FfiContext as *const FfiContext);

        let dbg = if dbg.is_null() { None } else { Some(&mut *dbg) };
        (&mut *core, &mut *ctx, dbg)
    }

    #[no_mangle]
    pub extern "C" fn tihle_z80_handle_read(ctx: *mut c_void, address: u16) -> u8 {
        let (core, emu, _dbg) = unsafe { Self::ctx_from_ptr(ctx) };

        emu.read_memory(core, address)
    }

    #[no_mangle]
    pub extern "C" fn tihle_z80_handle_instruction_read(ctx: *mut c_void, address: u16) -> u8 {
        let (core, emu, dbg) = unsafe { Self::ctx_from_ptr(ctx) };

        if dbg.map_or(false, |d| d.handle_instruction_fetch(address)) {
            core.request_yield();
            return 0;
        }
        emu.read_memory(core, address)
    }

    #[no_mangle]
    pub extern "C" fn tihle_z80_handle_write(ctx: ffi::Ctx, address: u16, value: u8) {
        let (core, emu, _dbg) = unsafe { Self::ctx_from_ptr(ctx) };

        emu.write_memory(core, address, value)
    }

    #[no_mangle]
    pub extern "C" fn tihle_z80_handle_port_read(ctx: ffi::Ctx, address: u16) -> u8 {
        let (core, emu, _dbg) = unsafe { Self::ctx_from_ptr(ctx) };

        emu.read_io(core, address as u8)
    }

    #[no_mangle]
    pub extern "C" fn tihle_z80_handle_port_write(ctx: ffi::Ctx, address: u16, value: u8) {
        let (core, emu, _dbg) = unsafe { Self::ctx_from_ptr(ctx) };

        emu.write_io(core, address as u8, value);
    }

    #[no_mangle]
    pub extern "C" fn tihle_z80_handle_trap(ctx: ffi::Ctx, trap_no: u16) -> usize {
        let (core, emu, _dbg) = unsafe { Self::ctx_from_ptr(ctx) };
        emu.trap(trap_no, core)
    }

    /// Run the core for the given number of cycles.
    ///
    /// The number of cycles to run for is inexact, because most instruction
    /// take more than one cycle to run; this function will return only on
    /// instruction boundaries.
    ///
    /// The provided `Ctx` is passed to traps for access to higher-level
    /// system state.
    pub fn run<D: DerefMut<Target = Debugger>>(
        &mut self,
        cycles: usize,
        ctx: &mut Emulator,
        dbg: Option<D>,
    ) -> usize {
        assert!(
            cycles > 0,
            "Running the CPU for zero cycles doesn't make sense"
        );
        // Safe: we pass mutable refs down into the core, which effecitvely
        // passes them back to callbacks by copying the parameter it receives;
        // there is no aliasing because it's just copying refs it has down to
        // a leaf function.
        let dbg = match dbg {
            None => ptr::null_mut(),
            Some(mut d) => d.deref_mut() as *mut _,
        };
        let ffi_ctx: FfiContext = (self as *mut Self, ctx as *mut Emulator, dbg);
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

    /// Request that the core yield back to the emulator at the next opportunity.
    ///
    /// This is usually on the memory fetch for the next instruction.
    pub fn request_yield(&mut self) {
        self.z80.yield_requested = true as u8;
    }
}
