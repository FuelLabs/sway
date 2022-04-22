library b512;
//! A wrapper around 2 b256 types to support the usage of 64-byte values in Sway, which are needed when working with public keys and signatures.

/// Stores two b256s in contiguous memory.
/// Guaranteed to be contiguous for use with ec-recover: std::ecr::ec_recover().
pub struct B512 {
    bytes: [b256;
    2],
}

// @todo use generic form when possible
pub trait From {
    fn from(h: b256, l: b256) -> Self;
} {
    // @todo add into() when tuples land, as it would probably return 2 b256 values
    // fn into() {...}
}

impl core::ops::Eq for B512 {
    fn eq(self, other: Self) -> bool {
        (self.bytes)[0] == (other.bytes)[0] && (self.bytes)[1] == (other.bytes)[1]
    }
}

/// Functions for casting between B512 and raw byte arrays.
impl From for B512 {
    fn from(h: b256, l: b256) -> B512 {
        B512 {
            bytes: [h,
            l], 
        }
    }
}

/// Methods on the B512 type
impl B512 {
    /// Initializes a new, zeroed B512.
    fn new() -> B512 {
        B512 {
            bytes: [0x0000000000000000000000000000000000000000000000000000000000000000,
            0x0000000000000000000000000000000000000000000000000000000000000000], 
        }
    }
}
