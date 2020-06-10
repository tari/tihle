use std::ops::Range;

pub struct Memory {
    ram: [u8; 0x8000],
    // Only supports one page for now
    flash: [u8; 0x4000],
}

const RAM_ADDRS: std::ops::RangeInclusive<u16> = 0x8000..=0xFFFF;
const BANKA_ADDRS: std::ops::Range<u16> = 0x4000..0x8000;

impl Memory {
    pub fn new(flash_contents: &[u8]) -> Self {
        let mut flash = [0; 0x4000];
        flash[..flash_contents.len()].copy_from_slice(flash_contents);
        Memory {
            ram: [0; 0x8000],
            flash,
        }
    }

    #[inline]
    pub fn read_u16(&mut self, addr: u16) -> u16 {
        if RAM_ADDRS.contains(&addr) || BANKA_ADDRS.contains(&addr) {
            (self[addr] as u16) | ((self[addr + 1] as u16) << 8)
        } else {
            panic!("Attempted to read unmapped memory at {:04X}", addr);
        }
    }

    #[inline]
    pub fn write_u16(&mut self, addr: u16, value: u16) {
        if RAM_ADDRS.contains(&addr) || BANKA_ADDRS.contains(&addr) {
            self[addr] = value as u8;
            self[addr + 1] = (value >> 8) as u8;
        } else {
            panic!("Attempted to write unmapped memory at {:04X}", addr);
        }

    }

    /// Checked memory read; returns None if `addr` is not in mapped memory.
    pub fn get(&self, addr: u16) -> Option<u8> {
        if RAM_ADDRS.contains(&addr) || BANKA_ADDRS.contains(&addr) {
            Some(self[addr])
        } else {
            None
        }
    }

    pub fn put(&mut self, addr: u16, value: u8) -> bool {
        if RAM_ADDRS.contains(&addr) || BANKA_ADDRS.contains(&addr) {
            self[addr] = value;
            false
        } else {
            true
        }
    }
}

impl std::ops::Index<u16> for Memory {
    type Output = u8;

    #[inline]
    fn index(&self, index: u16) -> &u8 {
        if BANKA_ADDRS.contains(&index) {
            &self.flash[(index - BANKA_ADDRS.start) as usize]
        } else if RAM_ADDRS.contains(&index) {
            &self.ram[(index - RAM_ADDRS.start()) as usize]
        } else {
            panic!("Attempted index into unmapped memory");
        }
    }
}

impl std::ops::IndexMut<u16> for Memory {
    #[inline]
    fn index_mut(&mut self, index: u16) -> &mut u8 {
        if BANKA_ADDRS.contains(&index) {
            &mut self.flash[(index - BANKA_ADDRS.start) as usize]
        } else if RAM_ADDRS.contains(&index) {
            &mut self.ram[(index - RAM_ADDRS.start()) as usize]
        } else {
            panic!("Attempted index into unmapped memory");
        }
    }
}

impl std::ops::Index<Range<u16>> for Memory {
    type Output = [u8];

    fn index(&self, index: Range<u16>) -> &[u8] {
        if BANKA_ADDRS.contains(&index.start) && BANKA_ADDRS.contains(&index.end) {
            &self.flash[(index.start - BANKA_ADDRS.start) as usize..(index.end - BANKA_ADDRS.start) as usize]
        } else if RAM_ADDRS.contains(&index.start) && RAM_ADDRS.contains(&index.end) {
            &self.ram[(index.start - RAM_ADDRS.start()) as usize..(index.end - RAM_ADDRS.start()) as usize]
        } else {
            panic!("Attempted slice indexing of unmapped memory or spanning memories");
        }
    }
}

impl std::ops::IndexMut<Range<u16>> for Memory {
    fn index_mut(&mut self, index: Range<u16>) -> &mut [u8] {
        if BANKA_ADDRS.contains(&index.start) && BANKA_ADDRS.contains(&index.end) {
            &mut self.flash[(index.start - BANKA_ADDRS.start) as usize..(index.end - BANKA_ADDRS.start) as usize]
        } else if RAM_ADDRS.contains(&index.start) && RAM_ADDRS.contains(&index.end) {
            &mut self.ram[(index.start - RAM_ADDRS.start()) as usize..(index.end - RAM_ADDRS.start()) as usize]
        } else {
            panic!("Attempted slice indexing of unmapped memory or spanning memories");
        }
    }
}
