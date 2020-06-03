
pub struct Display {
    buf: [[u8; 12]; 8],
}

impl Display {
    pub fn new() -> Self {
        Display {
            buf: [[0; 12]; 8]
        }
    }

    fn clear(&mut self) {
        unimplemented!()
    }
}
