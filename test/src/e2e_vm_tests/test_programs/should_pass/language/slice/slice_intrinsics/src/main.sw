script;

use utils::*;

fn alloc_slice<T>(len: u64) -> &mut __slice[T] {
    let size_in_bytes = len * __size_of::<T>();
    let ptr = asm(size_in_bytes: size_in_bytes) {
        aloc size_in_bytes;
        hp: raw_ptr
    };
    asm(buf: (ptr,  len)) {
        buf: &mut __slice[T]
    }
}

fn realloc_slice<T>(old: &mut __slice[T], len: u64) -> &mut __slice[T] {
    let old_ptr = old.ptr();
    let old_len_in_bytes = old.len() * __size_of::<T>();

    let new_len_in_bytes = len * __size_of::<T>();
    let new_ptr = asm(new_len_in_bytes: new_len_in_bytes, old_ptr: old_ptr, old_len_in_bytes: old_len_in_bytes) {
        aloc new_len_in_bytes;
        mcp hp old_ptr old_len_in_bytes;
        hp: raw_ptr
    };

    asm(buf: (new_ptr,  len)) {
        buf: &mut __slice[T]
    }
}

pub struct Vec<T> {
    buf: &mut __slice[T],
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

        let v: &mut T = __elem_at(self.buf, new_item_idx);

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

        let item: &mut T = __elem_at(self.buf, index);

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

fn type_check() {
    let immutable_array: [u64; 5] = [1, 2, 3, 4, 5];
    let _: &u64 = __elem_at(&immutable_array, 0);

    let mut mutable_array: [u64; 5] = [1, 2, 3, 4, 5];
    let _: &u64 = __elem_at(&mutable_array, 0);
    let _: &mut u64 = __elem_at(&mut mutable_array, 0);
    let _: &u64 = __elem_at(&mut mutable_array, 0);

    let immutable_slice: &__slice[u64] = __slice(&immutable_array, 0, 5);
    let _: &u64 = __elem_at(immutable_slice, 0);

    let mutable_slice: &mut __slice[u64] = __slice(&mut mutable_array, 0, 5);
    let _: &mut u64 = __elem_at(mutable_slice, 0);
    let _: &u64 = __elem_at(mutable_slice, 0);
}

fn main()  {
    type_check();

    // index arrays
    let some_array: [u64; 5] = [1, 2, 3, 4, 5];
    assert(1, *__elem_at(&some_array, 0));
    assert(2, *__elem_at(&some_array, 1));
    assert(3, *__elem_at(&some_array, 2));
    assert(4, *__elem_at(&some_array, 3));
    assert(5, *__elem_at(&some_array, 4));

    // slice arrays
    let some_slice: &__slice[u64] = __slice(&some_array, 0, 5);
    assert(1, *__elem_at(some_slice, 0));
    assert(2, *__elem_at(some_slice, 1));
    assert(3, *__elem_at(some_slice, 2));
    assert(4, *__elem_at(some_slice, 3));
    assert(5, *__elem_at(some_slice, 4));

    // slice another slice
    let another_slice: &__slice[u64] = __slice(some_slice, 1, 4);
    assert(2, *__elem_at(another_slice, 0));
    assert(3, *__elem_at(another_slice, 1));
    assert(4, *__elem_at(another_slice, 2));

    // Vec impl using slices
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

    //indices as expressions
    assert(2, *__elem_at(&some_array, v.get(0)));

    let _some_slice: &__slice[u64] = __slice(&some_array, v.get(0), v.get(4));
    assert(2, *__elem_at(some_slice, v.get(0)));
}
