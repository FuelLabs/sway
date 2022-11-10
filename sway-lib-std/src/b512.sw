library b512;
//! A wrapper around 2 `b256` types to support the usage of 64-byte values in Sway, which are needed when working with public keys and signatures.
use ::constants::ZERO_B256;
use ::convert::From;

/// Stores two `b256`s in contiguous memory.
/// Guaranteed to be contiguous for use with ec-recover: std::ecr::ec_recover().
pub struct B512 {
    bytes: [b256; 2],
}

impl core::ops::Eq for B512 {
    fn eq(self, other: Self) -> bool {
        (self.bytes)[0] == (other.bytes)[0] && (self.bytes)[1] == (other.bytes)[1]
    }
}

/// Functions for casting between `B512` and an array of 2 `b256`s.
impl From<(b256, b256)> for B512 {
    fn from(components: (b256, b256)) -> B512 {
        B512 {
            bytes: [components.0, components.1],
        }
    }

    fn into(self) -> (b256, b256) {
        ((self.bytes)[0], (self.bytes)[1])
    }
}

/// Methods on the `B512` type
impl B512 {
    /// Initializes a new, zeroed `B512`.
    fn new() -> B512 {
        B512 {
            bytes: [ZERO_B256, ZERO_B256],
        }
    }
}
