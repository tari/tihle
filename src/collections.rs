//! Fast collections for 16-bit addresses.
//!
//! These configure hashers to be perfectly deterministic over
//! the input data, with no randomization. This is potentially vulnerable to bad performance
//! against crafted input, but is much faster than the default RandomState.
//!
//! For the provided type aliases this is trivially safe against denial of service through hash
//! collisions because any keys that collide are by definition equal.
//!
//! Create instances through [with_hasher].

pub type AddressSet = std::collections::HashSet<u16, InsecureAddressHashBuilder>;
pub type AddressMap<T> = std::collections::HashMap<u16, T, InsecureAddressHashBuilder>;

pub struct InsecureAddressHashBuilder;
pub struct InsecureAddressHasher(u16);

impl std::default::Default for InsecureAddressHashBuilder {
    fn default() -> Self {
        InsecureAddressHashBuilder
    }
}

impl std::hash::BuildHasher for InsecureAddressHashBuilder {
    type Hasher = InsecureAddressHasher;

    fn build_hasher(&self) -> Self::Hasher {
        InsecureAddressHasher(0)
    }
}

impl std::hash::Hasher for InsecureAddressHasher {
    fn finish(&self) -> u64 {
        self.0 as u64
    }

    fn write(&mut self, bytes: &[u8]) {
        assert_eq!(
            2,
            bytes.len(),
            "InsecureAddressHasher only accepts 2-byte values"
        );
        self.0 = u16::from_ne_bytes([bytes[0], bytes[1]]);
    }
}
