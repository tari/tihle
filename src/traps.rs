use super::bcalls;
use crate::bcalls::{set_flag, test_flag};
use crate::include::tios;
use crate::{Emulator, Z80};

/// Defined emulator traps.
///
/// These values must match those used in the OS image; see os/tihle-os.inc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
pub enum Trap {
    /// Terminate emulation.
    Reset = 0,
    /// Set up for a paged subroutine call.
    ///
    /// See the implementation of `bcall_handler` on page 0 of the OS for details.
    RomCall = 1,
    RomCallReturn = 2,
    OsInterrupt = 3,

    DivHLBy10 = 0x400F,
    PutMap = 0x4501,
    PutC = 0x4504,
    DispHL = 0x4507,
    ClrLCDFull = 0x4540,
    HomeUp = 0x4558,
    VPutMap = 0x455e,
    GrBufCpy = 0x4860,
    MemSet = 0x4C33,

    PrintCpuState = 0xFFFF,
}

impl Trap {
    pub fn handle(&self, emu: &mut Emulator, core: &mut Z80) -> usize {
        use Trap::*;

        trace!("Servicing trap {:04X} ({:?})", *self as u16, *self);
        match *self {
            Reset => {
                emu.terminate.set(true);
                Emulator::FORCE_YIELD
            }
            RomCall => bcalls::bcall_trap(emu, core),
            RomCallReturn => bcalls::bcall_trap_return(emu, core),
            OsInterrupt => {
                // The OS interrupt does a few things, none of which we
                // implement right now:
                //  * Keyboard scanning (call KbdScan)
                //  * Run indicator
                //  * Set onInterrupt,(onFlags) if ON is pressed
                if test_flag(emu, core, tios::indicFlags, tios::indicOnly) {
                    // Stop if only supposed to animate the run indicator
                    return 200;
                }

                // Keyboard scanning: if more than one key is pressed don't set any,
                // otherwise write the scan code to (kbdScanCode) and set the kbdSCR flag.
                if let Some(k) = emu.keyboard.scan() {
                    set_flag(emu, core, tios::kbdFlags, tios::kbdSCR);
                    emu.mem[tios::kbdScanCode] = k as u8;
                }

                400 // :shrug:
            }

            DivHLBy10 => bcalls::util::DivHLBy10(emu, core),
            PutMap => bcalls::display::PutMap(emu, core),
            PutC => bcalls::display::PutC(emu, core),
            DispHL => bcalls::display::DispHL(emu, core),
            ClrLCDFull => bcalls::display::ClrLCDFull(emu),
            HomeUp => bcalls::display::HomeUp(emu),
            VPutMap => bcalls::display::VPutMap(emu, core),
            GrBufCpy => bcalls::display::GrBufCpy(emu),
            MemSet => bcalls::memory::MemSet(emu, core),
            PrintCpuState => {
                info!("{:#?}", core.regs());
                0
            }
        }
    }
}
