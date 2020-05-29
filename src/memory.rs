use std::ops::Range;

const RAM_ADDRS: std::ops::RangeInclusive<u16> = 0x8000..=0xFFFF;

pub struct Memory {
    ram: [u8; 0x8000],
}

impl Memory {
    pub fn new() -> Self {
        Memory { ram: [0; 0x8000] }
    }

    #[inline]
    pub fn read_u16(&mut self, addr: u16) -> u16 {
        if !RAM_ADDRS.contains(&addr) {
            warn!("Attempted to read non-RAM memory at {:#06x}", addr);
            return 0;
        }

        let base = (addr - RAM_ADDRS.start()) as usize;
        self.ram[base] as u16 | ((self.ram[base + 1] as u16) << 8)
    }

    #[inline]
    pub fn write_u16(&mut self, addr: u16, value: u16) {
        unimplemented!();
    }
}

impl std::ops::Index<u16> for Memory {
    type Output = u8;

    #[inline]
    fn index(&self, index: u16) -> &u8 {
        &self.ram[(index - RAM_ADDRS.start()) as usize]
    }
}

impl std::ops::IndexMut<u16> for Memory {
    #[inline]
    fn index_mut(&mut self, index: u16) -> &mut u8 {
        &mut self.ram[(index - RAM_ADDRS.start()) as usize]
    }
}

impl std::ops::Index<Range<u16>> for Memory {
    type Output = [u8];

    fn index(&self, index: Range<u16>) -> &[u8] {
        &self.ram
            [(index.start - RAM_ADDRS.start()) as usize..(index.end - RAM_ADDRS.start()) as usize]
    }
}

impl std::ops::IndexMut<Range<u16>> for Memory {
    fn index_mut(&mut self, index: Range<u16>) -> &mut [u8] {
        &mut self.ram
            [(index.start - RAM_ADDRS.start()) as usize..(index.end - RAM_ADDRS.start()) as usize]
    }
}
