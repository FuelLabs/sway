library raw_ptr;

impl raw_ptr {
    /// Returns `true` if the pointer is null.
    pub fn is_null(self) -> bool {
        let addr = asm(ptr: self) { ptr: u64 };
        __eq(addr, 0)
    }

    /// Gets the address of the pointer.
    pub fn addr(self) -> u64 {
        asm(ptr: self) { ptr: u64 }
    }

    /// Calculates the offset from the pointer.
    pub fn add(self, count: u64) -> raw_ptr {
        let addr = asm(ptr: self) { ptr: u64 };
        let addr = __add(addr, count);
        asm(ptr: addr) { ptr: raw_ptr }
    }

    /// Calculates the offset from the pointer.
    pub fn sub(self, count: u64) -> raw_ptr {
        let addr = asm(ptr: self) { ptr: u64 };
        let addr = __sub(addr, count);
        asm(ptr: addr) { ptr: raw_ptr }
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
    
    /// Copies `size` bytes from `self` to `dst`.
    pub fn copy_to(self, dst: raw_ptr, count: u64) {
        asm(dst: dst, src: self, count: count) {
            mcp dst src count;
        };
    }
    
    /// Copies `size` bytes from `src` to `self`.
    pub fn copy_from(self, src: raw_ptr, count: u64) {
        asm(dst: self, src: src, count: count) {
            mcp dst src count;
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
