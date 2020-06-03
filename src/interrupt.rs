//! The interrupt scheduler

use std::time::{Duration, Instant};

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
/// ~120 Hz with timer1 only. See [the table on WikiTI](
/// https://wikiti.brandonw.net/index.php?title=83Plus:Ports:04) for precise
/// values.
///
/// ## Implementation
///
/// Only the ON key and timer1 are currently implemented.
pub struct InterruptController {
    /// Timer 1 is normally enabled at about 120 Hz.
    timer1_period: Duration,
    timer1_last: Instant,
    timer1_enabled: bool,
    timer1_pending: bool,

    on_enabled: bool,
    on_pending: bool,
}

impl InterruptController {
    pub fn new() -> Self {
        InterruptController {
            timer1_period: Duration::from_nanos(1e9 as u64 / 140),
            timer1_last: Instant::now(),
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

    /// Poll for pending interrupts.
    ///
    /// The application should call this periodically, ideally at least as
    /// often as timer interrupts may fire.
    ///
    /// Returns whether any interrupts are pending (in which case the CPU IRQ
    /// line should be set), and the time until next interrupt, if known.
    pub fn poll(&mut self) -> (bool, Option<Duration>) {
        // Update timer
        let now = Instant::now();
        let since_last_timer = now.saturating_duration_since(self.timer1_last);
        if since_last_timer > self.timer1_period {
            self.timer1_pending = true;
            self.timer1_last = now;
        }

        let pending = self.timer1_pending || self.on_pending;
        let next = if self.timer1_enabled {
            // last + period if it's not in the past, otherwise now + period
            // as a lower bound for the next one.
            Some(
                self.timer1_period
                    .checked_sub(since_last_timer)
                    .unwrap_or(self.timer1_period),
            )
        } else {
            None
        };

        (pending, next)
    }
}
