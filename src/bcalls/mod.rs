use crate::{Emulator, Z80};

mod display;

pub fn bcall_trap(emu: &mut Emulator, core: &mut Z80) -> usize {
    // Vector is inline in the caller's code; read from the return target
    // address and update the return target to be past the vector.
    let sp = core.regs_mut().sp;
    let ret_addr = emu.mem.read_u16(sp);
    let bcall_addr = emu.mem.read_u16(ret_addr);
    emu.mem.write_u16(sp, ret_addr + 2);

    debug!("Trapping bcall {:04x}", bcall_addr);
    match bcall_addr {
        0x4501 => display::PutMap(emu, core),
        0x4504 => display::PutC(emu, core),
        0x450A => display::PutS(emu, core),
        0x4540 => display::ClrLCDFull(emu),
        0x4558 => display::HomeUp(emu),
        0x4860 => display::GrBufCpy(emu),
        _ => {
            warn!("Unhandled bcall: {:04x}", bcall_addr);
            0
        }
    }
}

pub fn test_flag(emu: &Emulator, core: &Z80, byte: u8, bit: u8) -> bool {
    (emu.mem
        .get(core.regs().iy.wrapping_add(byte as u16))
        .unwrap_or(0)
        & (1 << bit))
        != 0
}
