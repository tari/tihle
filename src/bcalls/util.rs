#![allow(non_snake_case)]

use crate::Z80;

pub fn DivHLBy10(cpu: &mut Z80) -> usize {
    let regs = cpu.regs_mut();

    let quotient = regs.hl / 10;
    let remainder = regs.hl % 10;
    regs.hl = quotient;
    regs.set_a(remainder as u8);

    160
}
