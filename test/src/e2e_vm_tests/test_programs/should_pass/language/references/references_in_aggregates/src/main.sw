script;

struct A {
    r_u8: &u8,
    r_array: &[u64;3],
}

impl A {
    fn new() -> Self {
        Self { r_u8: &0, r_array: &[0, 0, 0] }
    }
    
    fn use_me(self) {
        poke(self.r_u8);
        poke(self.r_array);
    }
}

struct B {
    r_a: &A,
    r_array: &[&A;3],
}

impl B {
    fn new() -> Self {
        let r_a = &A::new();
        Self { r_a: r_a, r_array: &[r_a, r_a, r_a] }
    }
    
    fn use_me(self) {
        poke(self.r_a);
        poke(self.r_array);
    }
}

#[inline(always)]
fn in_structs() {
    let x = 123u8;
    let array: [u64;3] = [111, 222, 333];

    let a = A { r_u8: &x, r_array: &array };
    let b = B { r_a: &a, r_array: &[&a, &a, &a] };

    let r_x_ptr = asm(r: &x) { r: raw_ptr };
    let r_array_ptr = asm(r: &array) { r: raw_ptr };

    let r_a_ptr = asm(r: &a) { r: raw_ptr };

    let a_r_u8_ptr = asm(r: a.r_u8) { r: raw_ptr };
    let a_r_array_ptr = asm(r: a.r_array) { r: raw_ptr };

    assert(r_x_ptr == a_r_u8_ptr);
    assert(r_array_ptr == a_r_array_ptr);

    let b_r_a_ptr = asm(r: b.r_a) { r: raw_ptr };

    assert(r_a_ptr == b_r_a_ptr);

    let a_r_u8_ptr_val = a_r_u8_ptr.read::<u8>();
    let a_r_array_ptr_val = a_r_array_ptr.read::<[u64;3]>();

    assert(a_r_u8_ptr_val == x);
    assert(a_r_array_ptr_val[0] == array[0]);
    assert(a_r_array_ptr_val[1] == array[1]);
    assert(a_r_array_ptr_val[2] == array[2]);

    let b_r_a_ptr_val = b_r_a_ptr.read::<A>();
    let a_r_u8_ptr_over_b = asm(r: b_r_a_ptr_val.r_u8) { r: raw_ptr };
    
    assert(a_r_u8_ptr_over_b == a_r_u8_ptr);
}

#[inline(never)]
fn in_structs_not_inlined() {
    in_structs()
}

#[inline(never)]
fn test_all_inlined() {
    in_structs();
}

#[inline(never)]
fn test_not_inlined() {
    in_structs_not_inlined();
}

fn main() -> u64 {
    test_all_inlined();
    test_not_inlined();

    A::new().use_me();
    B::new().use_me();

    42
}

fn poke<T>(_x: T) { }