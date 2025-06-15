library;

impl str {
    /// Return a `raw_ptr` to the beginning of the string slice.
    pub fn as_ptr(self) -> raw_ptr {
        let (ptr, _) = asm(s: self) {
            s: (raw_ptr, u64)
        };
        ptr
    }

    /// Return the length of the string slice in bytes.
    pub fn len(self) -> u64 {
        let (_, len) = asm(s: self) {
            s: (raw_ptr, u64)
        };
        len
    }
}

pub fn from_str_array<S>(s: S) -> str {
    __assert_is_str_array::<S>();
    let str_size = __size_of_str_array::<S>();
    let src = __addr_of(s);

    let ptr = asm(size: __size_of::<S>(), dest, src: src) {
        aloc size;
        move dest hp;
        mcp dest src size;
        dest: raw_ptr
    };

    asm(s: (ptr, str_size)) {
        s: str
    }
}
