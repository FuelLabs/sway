library;

impl<T> __ptr[T] {
    /// Calculates the offset from the pointer.
    pub fn add(self, count: u64) -> __ptr[T] {
        __ptr_add::<T>(self, count)
    }

    /// Calculates the offset from the pointer.
    pub fn sub(self, count: u64) -> __ptr[T] {
        __ptr_sub::<T>(self, count)
    }

    /// Reads the given type of value from the address.
    pub fn read<U>(self) -> U {
        if __is_reference_type::<U>() {
            asm(ptr: self) { ptr: U }
        } else {
            asm(ptr: self, val) {
                lw val ptr i0;
                val: U
            }
        }
    }

    /// Copies `count * size_of<T>` bytes from `self` to `dst`.
    pub fn copy_to(self, dst: __ptr[T], count: u64) {
        let len = __mul(count, __size_of::<T>());
        asm(dst: dst, src: self, len: len) {
            mcp dst src len;
        };
    }

    /// Writes the given value to the address.
    pub fn write<U>(self, val: U) {
        if __is_reference_type::<U>() {
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
