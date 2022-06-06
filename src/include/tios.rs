// Values match ti83plus.inc
#![allow(non_upper_case_globals)]

pub const kbdScanCode: u16 = 0x843F;

pub const curRow: u16 = 0x844B;
pub const curCol: u16 = 0x844C;

pub const OP1: u16 = 0x8478;

pub const textShadow: u16 = 0x8508;

pub const penCol: u16 = 0x86d7;
pub const penRow: u16 = 0x86d8;

pub const cmdShad: u16 = 0x966e;

pub const pTemp: u16 = 0x982E;
pub const progPtr: u16 = 0x9830;

pub const flags: u16 = 0x98f0;
pub const kbdFlags: u8 = 0;
pub const kbdSCR: u8 = 3;

pub const appFlags: u8 = 0xd;
pub const appAutoScroll: u8 = 2;
pub const indicFlags: u8 = 0x12;
pub const indicOnly: u8 = 2;

pub const sGrFlags: u8 = 0x14;
pub const grfSplit: u8 = 0;
pub const vertSplit: u8 = 1;
pub const grfSChanged: u8 = 2;
pub const grfSplitOverride: u8 = 3;
pub const textWrite: u8 = 7;

/// Primary graph buffer
pub const plotSScreen: u16 = 0x9340;

/// Fixed value, topmost byte of the symbol table.
pub const symTable: u16 = 0xFE66;
