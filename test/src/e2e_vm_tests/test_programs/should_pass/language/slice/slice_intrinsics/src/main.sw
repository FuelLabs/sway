script;

trait Log {
    fn log(self);
}

impl Log for str {
    fn log(self) {
        let encoded = encode(self);
        asm(ra: u64::max(), ptr: encoded.ptr(), len: encoded.len::<u8>()) {
            logd ra zero ptr len;
        }
    }
}


impl Log for u64 {
    fn log(self) {
        let encoded = encode(self);
        asm(ra: u64::max(), ptr: encoded.ptr(), len: encoded.len::<u8>()) {
           logd ra one ptr len;
        }
    }
}

struct NewLine {}

fn new_line() -> NewLine {
    NewLine { }
}

impl Log for NewLine {
    fn log(self) {
        asm(ra: u64::max(), rb: 2) {
            logd ra rb zero zero;
        }
    }
}

impl<A, B> Log for (A, B)
where
    A: Log,
    B: Log,
{
    fn log(self) {
        self.0.log();
        self.1.log();
    }
}

impl<A, B, C> Log for (A, B, C)
where
    A: Log,
    B: Log,
    C: Log,
{
    #[allow(dead_code)]
    fn log(self) {
        self.0.log();
        self.1.log();
        self.2.log();
    }
}

impl<A, B, C, D> Log for (A, B, C, D)
where
    A: Log,
    B: Log,
    C: Log,
    D: Log
{
    fn log(self) {
        self.0.log();
        self.1.log();
        self.2.log();
        self.3.log();
    }
}


impl<A, B, C, D, E> Log for (A, B, C, D, E)
where
    A: Log,
    B: Log,
    C: Log,
    D: Log,
    E: Log,
{
    fn log(self) {
        self.0.log();
        self.1.log();
        self.2.log();
        self.3.log();
        self.4.log();
    }
}

impl<T> Log for Vec<T> {
    fn log(self) {
        (
            "Vec",
            asm(v: self.buf.ptr()) { v: u64 },
            self.buf.len(),
            self.len,
        ).log();
    }
}

pub struct Vec<T> {
    buf: &__slice[T],
    len: u64,
}

impl<T> Vec<T> {
    pub fn new() -> Self {
        let ptr = asm() {
            hp: raw_ptr
        };
        Self {
            buf: asm(buf: (ptr,  0)) {
                buf: &__slice[T]
            },
            len: 0
        }
    }

    pub fn push(ref mut self, item: T) {
        ("Vec::push", new_line()).log();
        (self, new_line()).log();

        let new_item_idx = self.len;
        let current_cap = self.buf.len();
        if new_item_idx >= current_cap {
            let new_cap = if current_cap == 0 {
                1
            } else {
                current_cap * 2
            };
            let new_cap_in_bytes = new_cap * __size_of::<T>();

            let old_buf_ptr = self.buf.ptr();
            let old_cap_in_bytes = current_cap * __size_of::<T>();

            let ptr = asm(new_cap_in_bytes: new_cap_in_bytes, old_buf_ptr: old_buf_ptr, old_cap_in_bytes: old_cap_in_bytes) {
                aloc new_cap_in_bytes;
                mcp hp old_buf_ptr old_cap_in_bytes;
                hp: raw_ptr
            };

            self.buf = asm(buf: (ptr, new_cap)) {
                buf: &__slice[T]
            };
        }

        (self, new_line()).log();

        let v: &mut T = __slice_elem(self.buf, new_item_idx);
        ("elem", new_item_idx, " at ", asm(v: v) { v: u64 }, NewLine{}).log();
        *v = item;

        self.len += 1;
    }

    pub fn get(self, index: u64) -> T {
        ("Vec::get ", index).log();
        let item: &mut T = __slice_elem(self.buf, index);
        (asm(v: item) { v: u64 }, NewLine{}).log();
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
