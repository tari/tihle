use arr_macro::arr;

pub struct Display {
    /// Display buffer of one byte per pixel, LSb-only.
    ///
    /// This is hungrier for memory than packed 1bpp, but much easier to manipulate.
    buf: [u8; Self::ROWS * Self::COLS],

    /// X or Y auto-increment or -decrement.
    auto_address_mode: AutoAddressMode,
    /// 6- or 8-bit word mode
    word_mode: WordMode,
    /// Set when an address is written, and reset by any data read.
    ///
    /// When this is set, a dummy read is required to get data at the desired address
    /// so the address should not change on the first read (then this gets reset).
    /// This doesn't strictly match how the LCD driver is documented, but ought to be
    /// sufficient for most users.
    address_update_pending: bool,
    /// X address; this is the active row, because the LCD driver is strange.
    addr_x: u8,
    /// Y address; active word in the current row.
    addr_y: u8,
}

#[derive(Debug)]
enum AutoAddressMode {
    DecrementX,
    IncrementX,
    DecrementY,
    IncrementY,
}

#[derive(Debug)]
enum WordMode {
    Bit6,
    Bit8,
}

impl WordMode {
    /// Get the maximum Y address (inclusive) for the current word size.
    fn max_y_addr(&self) -> u8 {
        match *self {
            WordMode::Bit6 => 19,
            WordMode::Bit8 => 14,
        }
    }

    fn word_size(&self) -> usize {
        match *self {
            WordMode::Bit6 => 6,
            WordMode::Bit8 => 8,
        }
    }
}

pub enum ScrollDirection {
    /// Scroll the screen up, inserting blank lines at the bottom.
    Up,
    /// Scroll the screen down, inserting blank lines at the top.
    Down,
}

impl Display {
    pub fn new() -> Self {
        Display {
            buf: [0; Self::ROWS * Self::COLS],
            // TI-OS uses X-increment mode and leaves the LCD in 8-bit mode
            auto_address_mode: AutoAddressMode::IncrementX,
            word_mode: WordMode::Bit8,
            address_update_pending: true,
            addr_x: 0,
            addr_y: 0,
        }
    }

    pub fn get_buffer(&self) -> &[u8; Self::ROWS * Self::COLS] {
        &self.buf
    }

    pub fn get_pixel(&self, x: u8, y: u8) -> u8 {
        self.buf[y as usize * Self::COLS + x as usize]
    }

    fn get_pixel_mut(&mut self, x: u8, y: u8) -> &mut u8 {
        &mut self.buf[y as usize * Self::COLS + x as usize]
    }

    /// Get a mutable view of screen rows.
    ///
    /// This is often easier to work with than the linear buffer.
    #[inline]
    fn as_rows(&mut self) -> [&mut [u8]; Self::ROWS] {
        let mut out: [&mut [u8]; Self::ROWS] = arr![&mut []; 64];

        let mut tail = &mut self.buf[..];
        for row in 0..Self::ROWS {
            let split = tail.split_at_mut(Self::COLS);
            out[row] = split.0;
            tail = split.1;
        }

        debug_assert!(tail.is_empty());
        out
    }

    pub const ROWS: usize = 64;
    pub const COLS: usize = 96;

    pub fn clear(&mut self) {
        for byte in self.buf.iter_mut() {
            *byte = 0;
        }
    }

    pub fn scroll(&mut self, direction: ScrollDirection, count: usize) {
        if count == 0 {
            return;
        }
        if count >= Self::ROWS {
            self.clear();
            return;
        }
        let mut rows = self.as_rows();

        use ScrollDirection::*;
        match direction {
            Up => {
                for (src_idx, dst_idx) in (count..Self::ROWS).zip(0..Self::ROWS - count) {
                    let (dst, src) = rows.split_at_mut(src_idx);
                    dst[dst_idx].copy_from_slice(src[0]);
                }
                for clear_row in (Self::ROWS - count)..Self::ROWS {
                    for value in rows[clear_row].iter_mut() {
                        *value = 0;
                    }
                }
            }
            Down => {
                for (src_idx, dst_idx) in
                    (0..Self::ROWS - count).rev().zip((count..Self::ROWS).rev())
                {
                    let (src, dst) = rows.split_at_mut(dst_idx);
                    dst[0].copy_from_slice(src[src_idx]);
                }
                for clear_row in 0..count {
                    for value in rows[clear_row].iter_mut() {
                        *value = 0;
                    }
                }
            }
        }
    }

    /// Copy the given fullscreen image to the display.
    ///
    /// `data` must be 768 bytes.
    pub fn blit_fullscreen(&mut self, data: &[u8]) {
        assert_eq!(768, data.len());
        for (block, screen) in data
            .iter()
            .copied()
            .map(expand_byte)
            .zip(self.buf.chunks_exact_mut(8))
        {
            screen.copy_from_slice(&block.to_be_bytes());
        }
    }

    /// Blit data onto the screen with the given width in pixels, overwriting.
    pub fn blit_8bit_over(&mut self, x: u8, y: u8, data: &[u8], width: u8) {
        let row = y as usize;
        let col = x as usize;

        // Do nothing if offscreen
        if col > Self::COLS || row > Self::ROWS {
            return;
        }

        for (&data_row, screen_row) in data.iter().zip(&mut self.as_rows()[row..]) {
            // Mask to width bits, explode into bytes
            let expanded = expand_byte(data_row & (0xFF << (8 - width))).to_be_bytes();
            // Copy from data to the buffer, clipping right
            let clipped_width = std::cmp::min(col + width as usize, Self::COLS) - col;
            screen_row[col..col + clipped_width].copy_from_slice(&expanded[..clipped_width]);
        }
    }

    pub fn invert_pixel(&mut self, x: u8, y: u8) {
        *self.get_pixel_mut(x, y) ^= 1;
    }

    /// Ensure the current Y address is in range for the current addressing mode.
    fn clamp_y_addr(&mut self) {
        self.addr_y = std::cmp::min(self.addr_y, self.word_mode.max_y_addr());
    }

    pub fn write_control(&mut self, command: u8) {
        trace!("LCD command write {:02X}", command);
        let mut wrote_addr = false;
        match command {
            0x00 => {
                self.word_mode = WordMode::Bit6;
                self.clamp_y_addr();
            }
            0x01 => {
                self.word_mode = WordMode::Bit8;
            }
            0x04 => {
                self.auto_address_mode = AutoAddressMode::DecrementX;
            }
            0x05 => {
                self.auto_address_mode = AutoAddressMode::IncrementX;
            }
            0x06 => {
                self.auto_address_mode = AutoAddressMode::DecrementY;
            }
            0x07 => {
                self.auto_address_mode = AutoAddressMode::IncrementY;
            }
            y if y & 0xE0 == 0x20 => {
                self.addr_y = y & 0x1F;
                self.clamp_y_addr();
                wrote_addr = true;
                debug!("Set LCD Y addr = {}", self.addr_y);
            }
            x if x & 0xC0 == 0x80 => {
                // Set x address
                self.addr_x = x & 0x3F;
                wrote_addr = true;
                debug!("Set LCD X addr = {}", self.addr_x);
            }
            unimp => {
                warn!("Unimplemented LCD command {:#04x}", unimp);
            }
        }

        if wrote_addr {
            self.address_update_pending = true;
        }
    }

    pub fn read_status(&self) -> u8 {
        let busy = 0;
        let word_size = match self.word_mode {
            WordMode::Bit6 => 0,
            WordMode::Bit8 => 1,
        };
        let display_on = 1;
        let reset = 0;

        use AutoAddressMode::*;
        let axis_counter = match self.auto_address_mode {
            IncrementX | DecrementX => 0,
            IncrementY | DecrementY => 1,
        };
        let auto_increment = match self.auto_address_mode {
            DecrementX | DecrementY => 0,
            IncrementX | IncrementY => 1,
        };

        (busy << 7)
            | (word_size << 6)
            | (display_on << 5)
            | (reset << 4)
            | (axis_counter << 1)
            | auto_increment
    }

    /// Update X and Y addresses based on the current mode.
    fn do_autoaddressing(&mut self) {
        use AutoAddressMode::*;
        let max_y = self.word_mode.max_y_addr();
        match self.auto_address_mode {
            IncrementY => {
                self.addr_y = (self.addr_y + 1) % (max_y + 1);
            }
            DecrementY => {
                self.addr_y = self.addr_y.wrapping_sub(1);
                self.clamp_y_addr();
            }
            IncrementX => {
                self.addr_x = (self.addr_x + 1) % Self::ROWS as u8;
            }
            DecrementX => {
                self.addr_x = std::cmp::min(self.addr_x.wrapping_sub(1), Self::ROWS as u8 - 1);
            }
        };
    }

    pub fn write_data(&mut self, data: u8) {
        debug!(
            "LCD data write {:02X} to ({},{}) {:?} {:?}",
            data, self.addr_y, self.addr_x, self.word_mode, self.auto_address_mode,
        );

        let word_size = self.word_mode.word_size();
        if self.addr_y as usize * word_size >= Self::COLS {
            // Ignore writes outside the screen
            self.do_autoaddressing();
            return;
        }

        let bytes = expand_byte(data).to_be_bytes();
        // Drop LSbs of 6-bit word
        let bytes_write = &bytes[8 - word_size..];
        // Copy bits to buffer
        let buf_start = (self.addr_x as usize * Self::COLS) + (self.addr_y as usize * word_size);
        for (dst, &src) in self.buf[buf_start..].iter_mut().zip(bytes_write.iter()) {
            *dst = src;
        }

        self.do_autoaddressing();
    }

    pub fn read_data(&mut self) -> u8 {
        let word_size = self.word_mode.word_size();
        let out = if self.addr_y as usize * word_size >= Self::COLS {
            // Y address is outside the screen, just read 0.
            warn!("LCD data read out of bounds at Y={}", self.addr_y);
            0
        } else {
            // Copy bytes from the display buffer into a local array, right-aligning within
            // the current word size.
            let mut bytes = [0u8; 8];
            let buf_start =
                (Self::COLS * self.addr_x as usize) + (word_size * self.addr_y as usize);
            for (dst, &src) in bytes[8 - word_size..]
                .iter_mut()
                .zip(self.buf[buf_start..].iter())
            {
                *dst = src;
            }
            // Pack the array into a byte to return
            let out = pack_byte(u64::from_be_bytes(bytes));
            debug!(
                "LCD data read {:02X} from ({},{}) {:?} {:?}",
                out, self.addr_y, self.addr_x, self.word_mode, self.auto_address_mode,
            );

            out
        };

        if self.address_update_pending {
            // This is the dummy read following an address update
            self.address_update_pending = false;
        } else {
            self.do_autoaddressing();
        }

        out
    }
}

/// Scatter a 1bpp byte into a 1Bpp lsb-only value.
fn expand_byte(x: u8) -> u64 {
    let x = x as u64;
    // Simply shifting the value will never cause adjacent bits to overlap in the interesting
    // position of each because the interesting bits are always 8 bits apart, so we can do all
    // the shifts first and mask afterward, meaning the primary bottleneck is the number of shifts
    // the CPU can execute per cycle.
    ((x << 49) | (x << 42) | (x << 35) | (x << 28) | (x << 21) | (x << 14) | (x << 7) | x)
        & 0x0101010101010101
}

fn pack_byte(x: u64) -> u8 {
    debug_assert_eq!(0, x & 0xFEFEFEFEFEFEFEFE);

    (x as u8)
        | (x >> 7) as u8
        | (x >> 14) as u8
        | (x >> 21) as u8
        | (x >> 28) as u8
        | (x >> 35) as u8
        | (x >> 42) as u8
        | (x >> 49) as u8
}

#[cfg(test)]
mod tests {
    use super::{Display, ScrollDirection};

    #[quickcheck]
    fn expand_byte_expands(x: u8) {
        let expanded = super::expand_byte(x).to_le_bytes();

        for bit in 0..8 {
            let expected = (x >> bit) & 1;
            let actual = expanded[bit];
            assert_eq!(expected, actual, "Expanded {:08b} -> {:?}", x, expanded);
        }
    }

    #[quickcheck]
    fn expand_then_pack_is_lossless(x: u8) {
        assert_eq!(x, super::pack_byte(super::expand_byte(x)));
    }

    fn setup_scrolling() -> Display {
        let mut display = Display::new();
        // Make each row say its original index
        for (i, row) in display.as_rows().iter_mut().enumerate() {
            for byte in row.iter_mut() {
                *byte = i as u8;
            }
        }

        display
    }

    #[quickcheck]
    fn scroll_up(distance: usize) {
        let mut display = setup_scrolling();
        display.scroll(ScrollDirection::Up, distance);
        let clamped = std::cmp::min(distance, Display::ROWS);

        for (i, row) in display
            .as_rows()
            .iter()
            .enumerate()
            .take(Display::ROWS - clamped)
        {
            assert!(
                row.iter().all(|&b| b == (i + clamped) as u8),
                "Row {} should have value {}, found {}",
                i,
                i + clamped,
                row[0]
            );
        }
        for i in (Display::ROWS - clamped)..Display::ROWS {
            assert!(
                display.as_rows()[i].iter().all(|&b| b == 0),
                "Row {} should be all zeroes, found {}",
                i,
                display.as_rows()[i][0]
            );
        }
    }

    #[quickcheck]
    fn scroll_down(distance: usize) {
        let mut display = setup_scrolling();
        display.scroll(ScrollDirection::Down, distance);
        let clamped = std::cmp::min(distance, Display::ROWS);

        for (i, row) in display.as_rows().iter().enumerate().skip(clamped) {
            assert!(
                row.iter().all(|&b| b == (i - distance) as u8),
                "Row {} should have value {}, found {}",
                i,
                i - distance,
                row[0]
            );
        }
        for i in 0..clamped {
            assert!(
                display.as_rows()[i].iter().all(|&b| b == 0),
                "Row {} should be all zeroes, found {}",
                i,
                display.as_rows()[i][0]
            );
        }
    }
}
