//! Ion vectors
#![allow(non_upper_case_globals)]

use super::tios;

const lVectors: u16 = tios::cmdShad + 80;
pub const ionVersion: u16 = lVectors;
pub const ionRandom: u16 = lVectors + 3;
pub const ionPutSprite: u16 = lVectors + 6;
pub const ionLargeSprite: u16 = lVectors + 9;
pub const ionGetPixel: u16 = lVectors + 12;
pub const ionFastCopy: u16 = lVectors + 15;
pub const ionDetect: u16 = lVectors + 18;
pub const ionDecompress: u16 = lVectors + 21;
