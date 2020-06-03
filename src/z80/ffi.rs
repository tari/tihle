//! FFI bindings for third_party/redcode/Z80.h
#![allow(unused)]

use std::ffi::c_void;
use std::fmt::Formatter;
use std::mem;

pub type Ctx = *mut c_void;

#[repr(C)]
struct Z80Internal {
    halt: u8,
    irq: u8,
    nmi: u8,
    iff1: u8,
    iff2: u8,
    ei: u8,
    im: u8,
}

#[repr(C)]
pub struct State {
    // ZZ80State
    pub pc: u16,
    pub sp: u16,
    pub af: u16,
    pub bc: u16,
    pub de: u16,
    pub hl: u16,
    pub ix: u16,
    pub iy: u16,
    pub af_: u16,
    pub bc_: u16,
    pub de_: u16,
    pub hl_: u16,
    pub r: u8,
    pub i: u8,
    memptr: u16,
    internal: Z80Internal,
}

impl State {
    pub fn set_interrupt_enable(&mut self, enable: bool) {
        self.internal.ei = if enable { 1 } else { 0 };
    }

    pub fn set_im(&mut self, mode: u8) {
        assert!((0..=2).contains(&mode));
        self.internal.im = mode;
    }
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        static FLAGS: &[(u8, &'static str, &'static str)] = &[
            (0x80, "P", "M"),  // Sign
            (0x40, "NZ", "Z"), // Zero
            // Unused
            (0x10, "", "HC"), // Half carry
            // Unused
            (0x04, "PE", "PO"), // Parity
            // Add/sub
            (0x01, "NC", "C"), // Carry
        ];

        if f.alternate() {
            write!(f, "CPU registers:\n")?;
            write!(
                f,
                "PC  {:04X}    SP  {:04X}    Intr {:>3}\n",
                self.pc,
                self.sp,
                if self.internal.ei != 0 { "ON" } else { "OFF" }
            )?;
            write!(f, "Flags")?;
            for (mask, set, unset) in FLAGS {
                write!(
                    f,
                    " {:2}",
                    if self.af & (*mask as u16) == 0 {
                        set
                    } else {
                        unset
                    }
                )?;
            }
            write!(f, "    Mode {:3}", self.internal.im)?;
            write!(
                f,
                "\n A    {:02X}    R     {:02X}    I     {:02X}\n",
                self.af >> 8,
                self.r,
                self.i
            )?;
            write!(
                f,
                "BC  {:04X}    DE  {:04X}    HL  {:04X}\n",
                self.bc, self.de, self.hl
            )?;
            write!(
                f,
                "IX  {:04X}    IY  {:04X}    AF' {:04X}\n",
                self.ix, self.iy, self.af_
            )?;
            write!(
                f,
                "BC' {:04X}    DE' {:04X}    HL' {:04X}",
                self.bc, self.de, self.hl
            )
        } else {
            f.debug_struct("State")
                .field("pc", &self.pc)
                .field("sp", &self.sp)
                .field("af", &self.af)
                .field("bc", &self.bc)
                .field("de", &self.de)
                .field("hl", &self.hl)
                .field("ix", &self.ix)
                .field("iy", &self.iy)
                .field("af'", &self.af_)
                .field("bc'", &self.bc_)
                .field("de'", &self.de_)
                .field("hl'", &self.hl_)
                .field("r", &self.r)
                .field("i", &self.i)
                .finish()
        }
    }
}

#[repr(C)]
pub struct Z80 {
    pub cycles: usize,
    pub context: Ctx,
    pub read: Option<extern "C" fn(context: Ctx, address: u16) -> u8>,
    pub write: Option<extern "C" fn(context: Ctx, address: u16, value: u8)>,
    pub port_in: Option<extern "C" fn(context: Ctx, port: u16) -> u8>,
    pub port_out: Option<extern "C" fn(context: Ctx, port: u16, value: u8)>,
    pub int_data: Option<extern "C" fn(context: Ctx) -> u32>,
    pub halt: Option<extern "C" fn(context: Ctx, state: u8)>,
    pub regs: State,

    // Internal fields below
    pub r7: u8,
    pub xy: u16,
    pub data: u32,
}

impl Z80 {
    pub fn new() -> Self {
        unsafe { mem::zeroed() }
    }

    pub fn is_halted(&self) -> bool {
        self.regs.internal.halt != 0
    }
}

extern "C" {
    pub fn z80_power(object: *mut Z80, state: u8);
    pub fn z80_reset(object: *mut Z80);
    pub fn z80_run(object: *mut Z80, cycles: usize) -> usize;
    pub fn z80_nmi(object: *mut Z80);
    pub fn z80_int(object: *mut Z80, state: u8);
}
