library address;
//! A wrapper around the b256 type to help enhance type-safety.

/// The Address type, a struct wrappper around the inner `value`.
pub struct Address {
    value: b256,
}

// @todo make this generic when possible
pub trait From {
    fn from(b: b256) -> Self;
} {
    fn into(addr: Address) -> b256 {
        addr.value
    }
}

/// Functions for casting between the b256 and Address types.
impl From for Address {
    fn from(bits: b256) -> Address {
        let addr = asm(r1: bits, value) {
            move value sp; // move stack pointer to $value
            cfei i32; // extend call frame by 32 bytes to allocate more memory. now $value is pointing to blank, uninitialized (but allocated) memory
            mcpi value r1 i32;
            value: b256
        };
        Address {
            value: addr,
        }
    }
}
