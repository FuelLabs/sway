library;

impl str {
    /// Return a `raw_ptr` to the begining of the string slice on the heap
    pub fn as_ptr(self) -> raw_ptr {
        let (ptr, _) = asm(s: self) { s: (raw_ptr, u64) };
        ptr
    }
    /// Return the length of the string slice in bytes
    pub fn len(self) -> u64 {
        let (_, len) = asm(s: self) { s: (raw_ptr, u64) };
        len
    }
}
