script;

#[inline(never)]
fn f3() -> raw_slice {
    let ptr = asm(size: 0) {
        aloc size;
        hp: raw_ptr
    };
    __transmute::<(raw_ptr, u64), raw_slice>((ptr, 0))
}

fn main() {
    f3();
}
