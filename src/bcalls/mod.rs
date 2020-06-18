use crate::{Emulator, Z80};

pub mod display;
pub mod memory;

const VECTOR_TABLE_PAGE: u8 = 0x1B;

pub fn bcall_trap(emu: &mut Emulator, core: &mut Z80) -> usize {
    let regs = core.regs_mut();
    // Update return address to skip the inline vector table location
    let ret_addr = emu.mem.read_u16(regs.sp);
    emu.mem.write_u16(regs.sp, ret_addr + 2);
    // Get vector table location
    let bcall_addr = emu.mem.read_u16(ret_addr);

    // Read target from vector table
    assert!(
        bcall_addr >= 0x4000 && bcall_addr < 0x8000,
        "Non-OS bcalls are not currently supported"
    );
    let target_page = emu.mem.read_paged(VECTOR_TABLE_PAGE, bcall_addr);
    let target_addr = emu.mem.read_u16_paged(VECTOR_TABLE_PAGE, bcall_addr + 1);

    if target_page == 0 && target_addr == 0 {
        if cfg!(debug_assertions) {
            panic!("Unimplemented bcall: {:04X} {:#?}", bcall_addr, regs);
        } else {
            error!("Unimplemented bcall: {:04X} {:#?}", bcall_addr, regs);
        }
        // Return immediately
        regs.pc = emu.mem.read_u16(regs.sp);
        regs.sp += 2;
        return 60;
    }

    // Push current bank A page onto the stack
    let orig_page = emu.mem.get_bank_a_page();
    regs.sp -= 2;
    emu.mem[regs.sp + 1] = orig_page;

    // Push current PC to act as a call to the vector
    regs.sp -= 2;
    emu.mem.write_u16(regs.sp, regs.pc);

    // Jump to vector
    trace!(
        "bcall {:04X} from page {:02X} -> {:02X}:{:04X}",
        bcall_addr,
        orig_page,
        target_page,
        target_addr
    );
    emu.mem.set_bank_a_page(target_page);
    regs.pc = target_addr;

    // bcalls take around 820 cycles in overhead, randomly split
    // it into 700 for this trap and 120 for the return.
    700
}

pub fn bcall_trap_return(emu: &mut Emulator, core: &mut Z80) -> usize {
    let regs = core.regs_mut();

    // Restore bank A mapping
    let page = emu.mem[regs.sp + 1];
    regs.sp += 2;
    emu.mem.set_bank_a_page(page);

    // Return to caller
    regs.pc = emu.mem.read_u16(regs.sp);
    regs.sp += 2;
    120
}

pub fn test_flag(emu: &Emulator, core: &Z80, byte: u8, bit: u8) -> bool {
    (emu.mem[core.regs().iy.wrapping_add(byte as u16)] & (1 << bit)) != 0
}

pub fn set_flag(emu: &mut Emulator, core: &Z80, byte: u8, bit: u8) {
    emu.mem[core.regs().iy.wrapping_add(byte as u16)] |= 1 << bit;
}
