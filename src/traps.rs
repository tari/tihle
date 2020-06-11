use super::bcalls;
use crate::{Emulator, Z80};

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
pub enum Trap {
    Reset = 0,
    RomCall = 1,
    OsInterrupt = 2,
}

impl Trap {
    pub fn handle(&self, emu: &mut Emulator, core: &mut Z80) -> usize {
        use Trap::*;

        match *self {
            Reset => {
                emu.terminate.set(true);
                Emulator::FORCE_YIELD
            }
            RomCall => bcalls::bcall_trap(emu, core),
            OsInterrupt => unimplemented!(),
        }
    }
}
