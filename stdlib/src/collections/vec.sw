library vec;
use::ops:: * ;
use::marker::Sized;

/// If `RawVec` is exceeded during a push, the size of the vector is doubled
pub struct Vec<T> {
    buf: RawVec, len: u64
}

impl<T> Vec<T> where T: Sized {
    /// Creates a new empty vector with enough size for 1 element of type `T`.
    /// The size of the vector's underlying memory buffer doubles each time its
    /// capacity is exceeded.
    /// If you know how big the buffer should be, use `Vec::with_capacity` instead.
    fn new() -> Self {
        let item_size = ~T::size_of();
        Vec {
            buf: ~RawVec::new(item_size), len: 0 
        }
    }

    /// Initializes a new vector with a pre-determined capacity allocated.
    /// If you know the size the vector will end up being, you should initialize
    /// your vector with this method instead, to save on future `aloc` calls.
    fn with_capacity(capacity: u64) -> Self {
        let capacity_bytes = capacity.multiply(~T::size_of());
        Vec {
            buf: ~RawVec::new(capacity_bytes), len: 0 
        }
    }

    /// Push an item on to the end of the vector.
    fn push(self, item: T) {
        let size_of_item = ~T::size_of();
        // If this item would exceed the boundaries of the underlying buffer, we
        // need allocate a new, bigger buffer.
        if ((((self).len.multiply(size_of_item)).add(size_of_item)).greater_than(self.buf.size)) {
            let new_buf_size = (2).multiply(self.buf.size);
            let mut new_buf: RawVec = ~RawVec::new(new_buf_size);
            // copy the contents of the old buf to the new one
            let mut i = 0;
            while i.less_than(self.buf.size) {
                copy_buf(self.buf, new_buf);
                i = i.add(1);
            }
            // put the new item in the buf
            new_buf.put_item_at_index_unchecked(self.len.multiply(size_of_item), item);
            self.len = self.len.add(1);
            self.buf = new_buf;
        }
        else {
            // write T to self.len * size_of_item
            // increase self.len by one
            self.buf.put_item_at_index_unchecked(self.len.multiply(size_of_item), item);
            self.len = self.len.add(1);
        }
    }
}

/// Contains the `ptr` to the start of the vector and the current length of the vector
/// in bytes
/// Basically a wrapper over the return value of `aloc`
pub struct RawVec {
    ptr: u64, size: u64, 
}

impl RawVec {
    fn new(init_size_bytes: u64) -> Self {
        let ptr = asm(r1: init_size_bytes, r2) {
            aloc r1;
            // add one, since the hp points to the free byte right below the byte that was just allocated.
            addi r2 hp i1;
            r2: u64
        };

        RawVec {
            ptr: ptr, size: init_size_bytes, 
        }
    }

    /// Given a byte offset and an item, write that item to the byte offset.
    /// No safety checks are performed.
    fn put_item_at_index_unchecked<T>(self, index_in_bytes: u64, item: T) {
        asm(r1: self.ptr.add(index_in_bytes), r2: item) {
            // write `item` to `self.ptr` + `index_in_bytes`
            sw r1 r2 i0;
        }
    }
}

fn copy_buf(buf1: RawVec, buf2: RawVec) {
    let mut buf1_read_ptr = self.ptr;
    let mut buf2_write_ptr = buf2.ptr;
    while buf1_read_ptr < self.ptr.add(self.size) {
        asm(r1: buf1_read_ptr, r2: buf2_write_ptr) {
            mv r2 r2;
        };
        // increment by one word
        buf1_read_ptr = buf1_read_ptr.add(64);
        buf2_write_ptr = buf2_write_ptr.add(64);
    }
    // copy ptr + size from buf1 into buf2
    // TODO
}
