library b512;
//! A wrapper around 2 b256 types to support the usage of 64-byte values in Sway, which are needed when working with public keys and signatures.
use ::constants::ZERO_B256;

/// Stores two b256s in contiguous memory.
/// Guaranteed to be contiguous for use with ec-recover: std::ecr::ec_recover().
pub struct B512 {
    bytes: [b256; 2],
}

// TODO: use generic, centrally defined trait when possible
pub trait From {
    fn from(h: b256, l: b256) -> Self;
    fn into(self) -> (b256, b256);
}

impl core::ops::Eq for B512 {
    fn eq(self, other: Self) -> bool {
        (self.bytes)[0] == (other.bytes)[0] && (self.bytes)[1] == (other.bytes)[1]
    }
}

/// Functions for casting between B512 and raw byte arrays.
impl From for B512 {
    fn from(h: b256, l: b256) -> B512 {
        B512 { bytes: [h, l] }
    }

    fn into(self) -> (b256, b256) {
        ((self.bytes)[0], (self.bytes)[1], )
    }
}

/// Methods on the B512 type
impl B512 {
    /// Initializes a new, zeroed B512.
    fn new() -> B512 {
        B512 {
            bytes: [ZERO_B256, ZERO_B256, ],
        }
    }
}
