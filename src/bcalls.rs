use crate::{Emulator, Z80};

pub fn bcall_trap(emu: &mut Emulator, core: &mut Z80) -> usize {
    // Vector is inline in the caller's code; read from the return target
    // address and update the return target to be past the vector.
    let sp = core.regs().sp;
    let ret_addr = emu.mem.read_u16(sp);
    let bcall_addr = emu.mem.read_u16(ret_addr);
    emu.mem.write_u16(sp, ret_addr + 2);

    debug!("Trapping bcall {:04x}", bcall_addr);
    match bcall_addr {
        _ => {
            warn!("Unhandled bcall: {:04x}", bcall_addr);
            0
        }
    }
}
