// Values match ti83plus.inc
#![allow(non_upper_case_globals)]

pub const curRow: u16 = 0x844B;
pub const curCol: u16 = 0x844C;

pub const OP1: u16 = 0x8478;

pub const textShadow: u16 = 0x8508;

pub const penCol: u16 = 0x86d7;
pub const penRow: u16 = 0x86d8;

pub const cmdShad: u16 = 0x966e;
// NOT TI-OS, but in RAM: Ion vector table

pub const flags: u16 = 0x98f0;

pub const appFlags: u8 = 0xd;
pub const appAutoScroll: u8 = 2;

/// Primary graph buffer
pub const plotSScreen: u16 = 0x9340;
