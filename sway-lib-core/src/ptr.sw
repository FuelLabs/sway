library ptr;

impl raw_ptr {
    /// Offsets the pointer by the given number of bytes
    // NOTE: It seems that it's not currently possible to call this method from
    // libstd, so it's been temporarily reimplemented there under the name
    // mem::ptr_offset.
    pub fn offset(self, amount: u64) -> raw_ptr {
        asm(r1: self, r2: amount, r3) {
            add r3 r2 r1;
            r3: raw_ptr
        }
    }
}
