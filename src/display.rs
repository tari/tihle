use arr_macro::arr;

pub struct Display {
    /// Display buffer of one byte per pixel, LSb-only.
    ///
    /// This is hungrier for memory than packed 1bpp, but much easier to manipulate.
    buf: [u8; Self::ROWS * Self::COLS],
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
                for dst_row in (0..(Self::ROWS - count)).rev() {
                    let (dst, src) = rows.split_at_mut(dst_row);
                    dst[dst_row].copy_from_slice(src[count]);
                }
                for clear_row in (Self::ROWS - count)..Self::ROWS {
                    for value in rows[clear_row].iter_mut() {
                        *value = 0;
                    }
                }
            }
            Down => {
                for dst_row in count..Self::ROWS {
                    let (src, dst) = rows.split_at_mut(dst_row - 1);
                    dst[0].copy_from_slice(src[dst_row - count]);
                }
                for clear_row in 0..count {
                    for value in rows[clear_row].iter_mut() {
                        *value = 0;
                    }
                }
            }
        }
    }

    /// Blit data onto the screen with the given width in pixels, overwriting.
    pub fn blit_8bit_over(&mut self, x: u8, y: u8, data: &[u8], width: u8) {
        assert!(x as usize <= Self::COLS && y as usize <= Self::ROWS);

        let row = y as usize;
        let col = x as usize;
        for (&data_row, screen_row) in data.iter().zip(&mut self.as_rows()[row..row + data.len()]) {
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

#[cfg(test)]
mod tests {
    #[quickcheck]
    fn expand_byte_expands(x: u8) {
        let expanded = super::expand_byte(x).to_le_bytes();

        for bit in 0..8 {
            let expected = (x >> bit) & 1;
            let actual = expanded[bit];
            assert_eq!(expected, actual, "Expanded {:08b} -> {:?}", x, expanded);
        }
    }
}
