script;

impl Buffer {
    pub fn write<T>(self, val: T, offset: u64) {
        // removed for brevity
    }
}
fn main() -> u64 {
    // Create a clone of the struct
    let buf = Buffer {
        ptr: Pointer {
            loc: 0u64,
        },
        len: 0,
    };
    buf.write(true, 0);
    buf.write(42, __size_of::<bool>());
    43
}

struct Pointer {
    loc: u64,
}

pub struct Buffer {
    ptr: Pointer,
    len: u64,
}
