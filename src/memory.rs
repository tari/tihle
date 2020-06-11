use std::ops::Range;

pub struct Memory {
    page0: [u8; 0x4000],
    bank_a: [u8; 0x4000],
    ram: [u8; 0x8000],
}

const PAGE0_ADDRS: std::ops::Range<u16> = 0..0x4000;
const BANKA_ADDRS: std::ops::Range<u16> = 0x4000..0x8000;
const RAM_ADDRS: std::ops::RangeInclusive<u16> = 0x8000..=0xFFFF;

impl Memory {
    pub fn new(page_0_contents: &[u8], bank_a_contents: &[u8]) -> Self {
        let mut page0 = [0; 0x4000];
        page0[..page_0_contents.len()].copy_from_slice(page_0_contents);

        let mut bank_a = [0; 0x4000];
        bank_a[..bank_a_contents.len()].copy_from_slice(bank_a_contents);

        Memory {
            page0,
            bank_a,
            ram: [0; 0x8000],
        }
    }

    #[inline]
    pub fn read_u16(&mut self, addr: u16) -> u16 {
        (self[addr] as u16) | ((self[addr + 1] as u16) << 8)
    }

    #[inline]
    pub fn write_u16(&mut self, addr: u16, value: u16) {
        self[addr] = value as u8;
        self[addr + 1] = (value >> 8) as u8;
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
}

impl std::ops::Index<u16> for Memory {
    type Output = u8;

    #[inline]
    fn index(&self, index: u16) -> &u8 {
        if PAGE0_ADDRS.contains(&index) {
            &self.page0[(index - PAGE0_ADDRS.start) as usize]
        } else if BANKA_ADDRS.contains(&index) {
            &self.bank_a[(index - BANKA_ADDRS.start) as usize]
        } else if RAM_ADDRS.contains(&index) {
            &self.ram[(index - RAM_ADDRS.start()) as usize]
        } else {
            unreachable!();
        }
    }
}

impl std::ops::IndexMut<u16> for Memory {
    #[inline]
    fn index_mut(&mut self, index: u16) -> &mut u8 {
        if PAGE0_ADDRS.contains(&index) {
            &mut self.page0[(index - PAGE0_ADDRS.start) as usize]
        } else if BANKA_ADDRS.contains(&index) {
            &mut self.bank_a[(index - BANKA_ADDRS.start) as usize]
        } else if RAM_ADDRS.contains(&index) {
            &mut self.ram[(index - RAM_ADDRS.start()) as usize]
        } else {
            unreachable!();
        }
    }
}

impl std::ops::Index<Range<u16>> for Memory {
    type Output = [u8];

    fn index(&self, index: Range<u16>) -> &[u8] {
        if PAGE0_ADDRS.contains(&index.start) && PAGE0_ADDRS.contains(&index.end) {
            &self.page0[(index.start - PAGE0_ADDRS.start) as usize
                ..(index.end - PAGE0_ADDRS.start) as usize]
        } else if BANKA_ADDRS.contains(&index.start) && BANKA_ADDRS.contains(&index.end) {
            &self.bank_a[(index.start - BANKA_ADDRS.start) as usize
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

impl std::ops::IndexMut<Range<u16>> for Memory {
    fn index_mut(&mut self, index: Range<u16>) -> &mut [u8] {
        if PAGE0_ADDRS.contains(&index.start) && PAGE0_ADDRS.contains(&index.end) {
            &mut self.page0[(index.start - PAGE0_ADDRS.start) as usize
                ..(index.end - PAGE0_ADDRS.start) as usize]
        } else if BANKA_ADDRS.contains(&index.start) && BANKA_ADDRS.contains(&index.end) {
            &mut self.bank_a[(index.start - BANKA_ADDRS.start) as usize
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
