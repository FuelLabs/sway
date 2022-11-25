library raw_ptr;

impl raw_ptr {
    /// Returns `true` if the pointer is null.
    pub fn is_null(self) -> bool {
        let null_ptr = asm() { zero: raw_ptr };
        __eq(self, null_ptr)
    }

    /// Calculates the offset from the pointer.
    pub fn add<T>(self, count: u64) -> raw_ptr {
        __ptr_add::<T>(self, count)
    }

    /// Calculates the offset from the pointer.
    pub fn sub<T>(self, count: u64) -> raw_ptr {
        __ptr_sub::<T>(self, count)
    }

    /// Reads the given type of value from the address.
    pub fn read<T>(self) -> T {
        if __is_reference_type::<T>() {
            asm(ptr: self) { ptr: T }
        } else {
            asm(ptr: self, val) {
                lw val ptr i0;
                val: T
            }
        }
    }

    /// Copies `count * size_of<T>` bytes from `self` to `dst`.
    pub fn copy_to<T>(self, dst: raw_ptr, count: u64) {
        let len = __mul(count, __size_of::<T>());
        asm(dst: dst, src: self, len: len) {
            mcp dst src len;
        };
    }

    /// Writes the given value to the address.
    pub fn write<T>(self, val: T) {
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
}
