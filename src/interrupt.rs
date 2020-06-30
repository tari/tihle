//! The interrupt scheduler

use bitflags::bitflags;
use std::time::Duration;

/// Handles interrupts for the 83+.
///
/// ## Sources
///
/// There are four interrupts, controlled by ports 2, 3 and 4.
///
/// The interrupts map to bits in each port as so:
///  * Bit 0: ON key
///  * Bit 1: timer 1
///  * Bit 2: timer 2
///  * Bit 4: link port activity
///
/// Port 2 is IRQ acknowledgement, but not implemented because the
/// regular 83+ doesn't implement it.
///
/// Port 3 masks interrupts. When a bit is set, the corresponding interrupt
/// is enabled. These bits shadow IRQ bits; acknowledging an interrupt should
/// clear the corresponding enable bit then set it again. If not cleared, the
/// interrupt will not be acknowledged.
///
/// Port 4 reads interrupt status and writes various controls. The bit for
/// each interrupt reads as whether that interrupt is pending.
///
/// On write, bits 1 and 2 adjust the timer frequencies, where the default is
/// ~118 Hz with timer1 only. See [the table on WikiTI](
/// https://wikiti.brandonw.net/index.php?title=83Plus:Ports:04) for precise
/// values.
///
/// ## Implementation
///
/// Only the ON key and timer1 are currently implemented.
#[derive(Debug)]
pub struct InterruptController {
    /// Timer 1 is normally enabled at about 120 Hz.
    timer1_period: Duration,
    timer1_remaining: Duration,
    timer1_enabled: bool,
    timer1_pending: bool,

    on_enabled: bool,
    on_pending: bool,
}

impl InterruptController {
    pub fn new() -> Self {
        let timer1_period = Duration::from_nanos(1e9 as u64 / 118);

        InterruptController {
            timer1_period,
            timer1_remaining: timer1_period,
            timer1_enabled: true,
            timer1_pending: false,

            on_enabled: true,
            on_pending: false,
        }
    }

    pub fn on_pressed(&mut self) {
        self.on_pending = true;
    }

    pub fn is_pending(&self) -> bool {
        self.timer1_pending || self.on_pending
    }

    /// Update timers as if the system has run for the given duration.
    ///
    /// This function drives timers and should be called after any CPU run.
    pub fn advance(&mut self, duration: Duration) {
        debug!("Advance timers {:?}", duration);
        if duration >= self.timer1_remaining {
            // Interrupt becomes pending
            self.timer1_pending = true;
            debug!("Timer1 interrupt fires");

            if duration >= (self.timer1_remaining + self.timer1_period) {
                warn!("Timer step of {:?} overflowed and skipped timer interrupts", duration);
                self.timer1_remaining = self.timer1_period;
            } else {
                self.timer1_remaining = self.timer1_period + self.timer1_remaining - duration;
            }
        } else {
            self.timer1_remaining -= duration;
        }
    }

    /// Poll for pending interrupts.
    ///
    /// The application should call this periodically, ideally at least as
    /// often as timer interrupts may fire.
    ///
    /// Returns whether any interrupts are pending (in which case the CPU IRQ
    /// line should be set), and the time until next interrupt, if known.
    pub fn poll(&mut self) -> (bool, Option<Duration>) {
        let pending =
            (self.timer1_pending && self.timer1_enabled) || (self.on_pending && self.on_enabled);
        let next = if self.timer1_enabled {
            Some(self.timer1_remaining)
        } else {
            None
        };

        trace!(
            "Interrupt controller polled: IRQ pending={}, next timer in {:?}",
            pending,
            next
        );
        (pending, next)
    }

    /// Read port 3, returning the enable status of interrupts.
    pub fn read_mask_port(&self) -> u8 {
        let mut out = InterruptFlags::empty();
        if self.on_enabled {
            out |= InterruptFlags::ON;
        }
        if self.timer1_enabled {
            out |= InterruptFlags::TIMER1;
        }

        out.bits()
    }

    /// Write port 3, setting the enable status (and pending flags).
    pub fn write_mask_port(&mut self, value: u8) {
        let value = InterruptFlags::from_bits_truncate(value);

        self.on_enabled = value.contains(InterruptFlags::ON);
        self.on_pending &= self.on_enabled;

        self.timer1_enabled = value.contains(InterruptFlags::TIMER1);
        self.timer1_pending &= self.timer1_enabled;
        trace!(
            "Wrote port 3; timer1 enabled={} pending={}",
            self.timer1_enabled,
            self.timer1_pending
        );
    }

    // Read port 4, getting the pending flags.
    pub fn read_status_port(&mut self) -> u8 {
        let mut out = InterruptFlags::empty();
        if self.on_pending {
            out |= InterruptFlags::ON;
        }
        if self.timer1_pending {
            out |= InterruptFlags::TIMER1;
        }

        out.bits()
    }
}

bitflags! {
    struct InterruptFlags: u8 {
        const ON = 0x01;
        const TIMER1 = 0x02;
        const TIMER2 = 0x04;
        const LINKPORT = 0x10;
    }
}
