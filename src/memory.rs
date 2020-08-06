use std::ops::Range;

/// Number of flash pages that exist. Must be a power of two.
const FLASH_PAGES: u8 = 0x20;

/// Emulator memory.
///
/// This struct controls the memory map and access to various memories. It
/// supports indexing with u16 to perform memory accesses, and range indexing
/// as long as addresses do not span multiple memory banks.
pub struct Memory {
    flash: Box<[[u8; 0x4000]]>,
    ram: [u8; 0x8000],
    // Page currently mapped into memory bank A
    bank_a_page: u8,
}

const PAGE0_ADDRS: std::ops::Range<u16> = 0..0x4000;
const BANKA_ADDRS: std::ops::Range<u16> = 0x4000..0x8000;
const RAM_ADDRS: std::ops::RangeInclusive<u16> = 0x8000..=0xFFFF;

impl Memory {
    pub fn new<P: AsRef<[u8]>, I: IntoIterator<Item = (u8, P)>>(flash_pages: I) -> Self {
        let mut flash: Box<_> = vec![[0u8; 0x4000]; FLASH_PAGES as usize].into_boxed_slice();
        for (page, contents) in flash_pages {
            let contents = contents.as_ref();
            flash[page as usize][..contents.len()].copy_from_slice(contents);
        }

        // Fill RAM with pseudo-random values
        let mut ram = [0; 0x8000];
        for (ram, value) in ram.iter_mut().zip((0u8..=0xFF).cycle()) {
            *ram = value;
        }

        Memory {
            flash,
            ram,
            bank_a_page: 0,
        }
    }

    #[inline]
    pub fn read_u16(&self, addr: u16) -> u16 {
        (self[addr] as u16) | ((self[addr + 1] as u16) << 8)
    }

    #[inline]
    pub fn write_u16(&mut self, addr: u16, value: u16) {
        self[addr] = value as u8;
        self[addr + 1] = (value >> 8) as u8;
    }

    /// Read a byte from memory bank A, in the given page.
    ///
    /// The given address must be in bank A (0x4000-0x8000). The read byte
    /// will come from the given memory page as if it were mapped into bank A.
    pub fn read_paged(&self, page: u8, addr: u16) -> u8 {
        assert!(
            BANKA_ADDRS.contains(&addr),
            "Paged read must refer to addresses in memory bank A"
        );
        self.flash[page as usize][(addr - BANKA_ADDRS.start) as usize]
    }

    /// Read a 16-bit value like [read_paged].
    pub fn read_u16_paged(&self, page: u8, addr: u16) -> u16 {
        (self.read_paged(page, addr) as u16) | ((self.read_paged(page, addr + 1) as u16) << 8)
    }

    /// Checked memory write.
    ///
    /// Fails if the given address refers to read-only memory.
    pub fn put(&mut self, addr: u16, value: u8) -> Result<(), ()> {
        if RAM_ADDRS.contains(&addr) {
            self[addr] = value;
            Ok(())
        } else {
            warn!(
                "Ignored write of byte {:02X} to read-only memory at {:04X}",
                value, addr
            );
            Err(())
        }
    }

    /// Get the current page mapped into bank A.
    pub fn get_bank_a_page(&self) -> u8 {
        self.bank_a_page
    }

    /// Set the page mapped into bank A.
    pub fn set_bank_a_page(&mut self, value: u8) {
        assert!(FLASH_PAGES.is_power_of_two());
        self.bank_a_page = value & (FLASH_PAGES - 1)
    }
}

/// Access a single byte of memory.
impl std::ops::Index<u16> for Memory {
    type Output = u8;

    #[inline]
    fn index(&self, index: u16) -> &u8 {
        if PAGE0_ADDRS.contains(&index) {
            &self.flash[0][(index - PAGE0_ADDRS.start) as usize]
        } else if BANKA_ADDRS.contains(&index) {
            &self.flash[self.bank_a_page as usize][(index - BANKA_ADDRS.start) as usize]
        } else if RAM_ADDRS.contains(&index) {
            &self.ram[(index - RAM_ADDRS.start()) as usize]
        } else {
            unreachable!();
        }
    }
}

/// Mutably access a single byte of memory.
impl std::ops::IndexMut<u16> for Memory {
    #[inline]
    fn index_mut(&mut self, index: u16) -> &mut u8 {
        if PAGE0_ADDRS.contains(&index) {
            &mut self.flash[0][(index - PAGE0_ADDRS.start) as usize]
        } else if BANKA_ADDRS.contains(&index) {
            &mut self.flash[self.bank_a_page as usize][(index - BANKA_ADDRS.start) as usize]
        } else if RAM_ADDRS.contains(&index) {
            &mut self.ram[(index - RAM_ADDRS.start()) as usize]
        } else {
            unreachable!();
        }
    }
}

/// Access a range of memory.
///
/// Panics if the chosen range spans memory banks, because multiple banks
/// cannot reliably be included in a single slice.
impl std::ops::Index<Range<u16>> for Memory {
    type Output = [u8];

    fn index(&self, index: Range<u16>) -> &[u8] {
        if PAGE0_ADDRS.contains(&index.start) && PAGE0_ADDRS.contains(&index.end) {
            &self.flash[0][(index.start - PAGE0_ADDRS.start) as usize
                ..(index.end - PAGE0_ADDRS.start) as usize]
        } else if BANKA_ADDRS.contains(&index.start) && BANKA_ADDRS.contains(&index.end) {
            &self.flash[self.bank_a_page as usize][(index.start - BANKA_ADDRS.start) as usize
                ..(index.end - BANKA_ADDRS.start) as usize]
        } else if RAM_ADDRS.contains(&index.start) && RAM_ADDRS.contains(&index.end) {
            &self.ram[(index.start - RAM_ADDRS.start()) as usize
                ..(index.end - RAM_ADDRS.start()) as usize]
        } else {
            panic!(
                "Attempted slice indexing of unmapped memory or spanning memories: {:?}",
                index
            );
        }
    }
}

/// Mutably access a range of memory.
///
/// Panics if the chosen range spans memory banks, because multiple banks
/// cannot reliably be included in a single slice.
impl std::ops::IndexMut<Range<u16>> for Memory {
    fn index_mut(&mut self, index: Range<u16>) -> &mut [u8] {
        if PAGE0_ADDRS.contains(&index.start) && PAGE0_ADDRS.contains(&index.end) {
            &mut self.flash[0][(index.start - PAGE0_ADDRS.start) as usize
                ..(index.end - PAGE0_ADDRS.start) as usize]
        } else if BANKA_ADDRS.contains(&index.start) && BANKA_ADDRS.contains(&index.end) {
            &mut self.flash[self.bank_a_page as usize][(index.start - BANKA_ADDRS.start) as usize
                ..(index.end - BANKA_ADDRS.start) as usize]
        } else if RAM_ADDRS.contains(&index.start) && RAM_ADDRS.contains(&index.end) {
            &mut self.ram[(index.start - RAM_ADDRS.start()) as usize
                ..(index.end - RAM_ADDRS.start()) as usize]
        } else {
            panic!(
                "Attempted slice indexing of unmapped memory or spanning memories: {:?}",
                index
            );
        }
    }
}
