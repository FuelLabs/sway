script;

use utils::*;

fn alloc_slice<T>(len: u64) -> &__slice[T] {
    let size_in_bytes = len * __size_of::<T>();
    let ptr = asm(size_in_bytes: size_in_bytes) {
        aloc size_in_bytes;
        hp: raw_ptr
    };
    asm(buf: (ptr,  len)) {
        buf: &__slice[T]
    }
}

fn realloc_slice<T>(old: &__slice[T], len: u64) -> &__slice[T] {
    let old_ptr = old.ptr();
    let old_len_in_bytes = old.len() * __size_of::<T>();

    let new_len_in_bytes = len * __size_of::<T>();
    let new_ptr = asm(new_len_in_bytes: new_len_in_bytes, old_ptr: old_ptr, old_len_in_bytes: old_len_in_bytes) {
        aloc new_len_in_bytes;
        mcp hp old_ptr old_len_in_bytes;
        hp: raw_ptr
    };

    asm(buf: (new_ptr,  len)) {
        buf: &__slice[T]
    }
}

pub struct Vec<T> {
    buf: &__slice[T],
    len: u64,
}

impl<T> Dbg for Vec<T> {
    fn dbg(self) {
        (
            "Vec { buf: (ptr: ",
            asm(v: self.buf.ptr()) { v: u64 },
            ", len: ",
            self.buf.len(),
            "), len: ",
            self.len,
            ")"
        ).dbg();
    }
}


impl<T> Vec<T> {
    pub fn new() -> Self {
        Self {
            buf: alloc_slice::<T>(0),
            len: 0
        }
    }

    pub fn push(ref mut self, item: T) {
        "Vec::push(...)".dbgln();
        ("    ", self).dbgln();

        let new_item_idx = self.len;
        let current_cap = self.buf.len();
        if new_item_idx >= current_cap {
            let new_cap = if current_cap == 0 {
                1
            } else {
                current_cap * 2
            };
            self.buf = realloc_slice(self.buf, new_cap);
            ("    After realloc: ", self).dbgln();
        }

        let v: &mut T = __slice_elem(self.buf, new_item_idx);

        let buffer_addr = asm(v: self.buf.ptr()) { v: u64 };
        let elem_addr = asm(v: v) { v: u64 };
        ("    elem ", new_item_idx, " at ", elem_addr, " buffer offset (in bytes): ", elem_addr - buffer_addr).dbgln();
        *v = item;

        self.len += 1;

        ("    ", self).dbgln();
    }

    pub fn get(self, index: u64) -> T {
        ("Vec::get(", index, ")").dbgln();
        ("    ", self).dbgln();

        let item: &mut T = __slice_elem(self.buf, index);

        let buffer_addr = asm(v: self.buf.ptr()) { v: u64 };
        let elem_addr = asm(v: item) { v: u64 };
        ("    element ", index, " at ", elem_addr, " buffer offset (in bytes): ", elem_addr - buffer_addr).dbgln();

        *item
    }
}

fn assert<T>(l: T, r: T)
where
    T: Eq + AbiEncode
{
    if l != r {
        __log(l);
        __log(r);
        __revert(1)
    }
}

fn main()  {
    let mut v: Vec<u64> = Vec::new();
    v.push(1);
    assert(v.get(0), 1);

    v.push(2);
    v.push(3);
    assert(v.get(0), 1);
    assert(v.get(1), 2);
    assert(v.get(2), 3);

    v.push(4);
    v.push(5);
    v.push(6);
    v.push(7);
    assert(v.get(0), 1);
    assert(v.get(1), 2);
    assert(v.get(2), 3);
    assert(v.get(3), 4);
    assert(v.get(4), 5);
    assert(v.get(5), 6);
    assert(v.get(6), 7);
}
