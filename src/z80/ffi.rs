//! FFI bindings for third_party/redcode/Z80.h
#![allow(unused)]

use std::ffi::c_void;
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
