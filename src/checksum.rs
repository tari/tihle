use std::io::{BufRead, Read, Result as IoResult};

pub(crate) struct ChecksumRead<R: Read> {
    pub r: R,
    pub sum: u16,
    pub bytes_read: usize,
}

impl<R: Read> Read for ChecksumRead<R> {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        let n = self.r.read(buf)?;

        self.bytes_read += n;
        self.sum = buf[..n]
            .iter()
            .fold(self.sum, |a, &x| a.wrapping_add(x as u16));
        Ok(n)
    }
}

impl<R: BufRead> BufRead for ChecksumRead<R> {
    fn fill_buf(&mut self) -> IoResult<&[u8]> {
        self.r.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        if let Ok(buf) = self.r.fill_buf() {
            self.sum = buf[..amt]
                .iter()
                .fold(self.sum, |a, &x| a.wrapping_add(x as u16));
        }
        self.bytes_read += amt;

        self.r.consume(amt)
    }
}

impl<R: Read> std::convert::From<R> for ChecksumRead<R> {
    fn from(r: R) -> Self {
        ChecksumRead {
            r,
            sum: 0,
            bytes_read: 0,
        }
    }
}
