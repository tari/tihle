#![allow(non_snake_case)]

use crate::{Emulator, Z80};

pub fn MemSet(emu: &mut Emulator, core: &mut Z80) -> usize {
    let sz = core.regs().bc;
    let ptr = core.regs().hl;
    let value = core.regs().get_a();

    for ofs in 0..sz {
        let _ = emu.mem.put(ptr.wrapping_add(ofs), value);
    }
    sz as usize * 16
}
