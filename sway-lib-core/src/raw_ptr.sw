library;

impl raw_ptr {
    /// Returns `true` if the pointer is null.
    pub fn is_null(self) -> bool {
        let null_ptr = asm() { zero: raw_ptr };
        __eq(self, null_ptr)
    }

    /// Calculates the offset from the pointer.
    pub fn add(self, offset: u64) -> Self {
        asm(ptr: self, offset: offset, new_ptr) {
            add new_ptr ptr offset;
            new_ptr: raw_ptr
        }
    }

    /// Calculates the offset from the pointer.
    pub fn sub(self, offset: u64) -> Self {
        asm(ptr: self, offset: offset, new_ptr) {
            sub new_ptr ptr offset;
            new_ptr: raw_ptr
        }
    }

    /// Writes the given byte to the address.
    pub fn write(self, val: u8) {
        let val_ptr = asm(r1: val) { r1: raw_ptr };
        asm(ptr: self, val: val_ptr) {
            sb ptr val i0;
        };
    }

    /// Reads a byte from the given address.
    pub fn read(self) -> u8 {
        asm(r1: self, r2) {
            lb r2 r1 i0;
            r2: u8
        }
    }

    /// Copies `count` bytes from `self` to `dst`.
    pub fn copy_to(self, ref mut dst: Self, count: u64) {
        asm(dst: dst, src: self, len: count) {
            mcp dst src len;
        };
    }

    /// Copies `count` bytes from `src` to `self`.
    pub fn copy_from(ref mut self, src: Self, count: u64) {
        asm(dst: self, src: src, len: count) {
            mcp dst src len;
        };
    }
}

impl raw_ptr {
    /// Calculates the offset from the pointer.
    pub fn add_t<T>(self, count: u64) -> Self {
        __ptr_add::<T>(self, count)
    }

    /// Calculates the offset from the pointer.
    pub fn sub_t<T>(self, count: u64) -> Self {
        __ptr_sub::<T>(self, count)
    }

    /// Reads the given type of value from the address.
    pub fn read_t<T>(self) -> T {
        if __is_reference_type::<T>() {
            asm(ptr: self) { ptr: T }
        } else {
            asm(ptr: self, val) {
                lw val ptr i0;
                val: T
            }
        }
    }

    /// Writes the given value to the address.
    pub fn write_t<T>(ref mut self, val: T) {
        if __is_reference_type::<T>() {
            asm(dst: self, src: val, count: __size_of_val(val)) {
                mcp dst src count;
            };
        } else {
            asm(ptr: self, val: val) {
                sw ptr val i0;
            };
        }
    }

    /// Copies `count * size_of<T>` bytes from `self` from `dst`.
    pub fn copy_to_t<T>(self, ref mut dst: Self, count: u64) {
        let len = __mul(count, __size_of::<T>());
        self.copy_to(dst, len);
    }

    /// Copies `count * size_of<T>` bytes from `src` to `self`.
    pub fn copy_from_t<T>(ref mut self, src: Self) {
        let len = __size_of::<T>();
        self.copy_from(src, len);
    }
}
