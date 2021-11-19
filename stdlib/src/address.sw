library address;

// @todo consider using tuple structs if they land.
// ie: pub struct Address(b256);
// let addr = Address(0x8900c5bec4ca97d4febf9ceb4754a60d782abbf3cd815836c1872116f203f861);
// usage:
pub struct Address {
    inner: b256,
}

// @todo make this generic when possible
pub trait From {
    fn from(b: b256) -> Self;
} {
    fn into(addr: Address) -> b256 {
        addr.inner
    }
}

impl From for Address {
    fn from(bits: b256) -> Address {
        let addr = asm(r1: bits, inner) {
            move inner sp; // move stack pointer to inner
            cfei i32; // extend call frame by 32 bytes to allocate more memory. now $inner is pointing to blank, uninitialized (but allocated) memory
            mcpi inner r1 i32; // refactor to use mcpi when implemented!
            inner: b256
        };
        Address {
            inner: addr,
        }
    }
}
