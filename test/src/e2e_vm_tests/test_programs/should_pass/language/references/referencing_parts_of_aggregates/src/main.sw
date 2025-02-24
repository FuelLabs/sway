script;

struct A {
    x: u8,
    r_x: &u8,
}

struct B {
    a: A,
    r_a: &A,
}

struct C {
    b: B,
    r_b: &B,
}

impl A {
    fn new() -> Self {
        Self { x: 0, r_x: &0 }
    }
    
    fn use_me(self) {
        poke(self.x);
        poke(self.r_x);
    }
}

impl B {
    fn new() -> Self {
        Self { a: A::new(), r_a: &A::new() }
    }
    
    fn use_me(self) {
        poke(self.a);
        poke(self.r_a);
    }
}

impl C {
    fn new() -> Self {
        Self { b: B::new(), r_b: &B::new() }
    }
    
    fn use_me(self) {
        poke(self.b);
        poke(self.r_b);
    }
}

// TODO: (REFERENCES) Add tests for accessing via reference chains once dereferencing operator `.` is implemented.
// TODO: (REFERENCES) Add tests for references to mutable parts of aggregates once reassignment is implemented.

#[inline(always)]
fn struct_fields() {
    let mut x = 123u8;

    let a = A { x, r_x: &x };
    let b = B { a, r_a: &a };
    let c = C { b, r_b: &b };

    let r_a_x: &u8 = &a.x;
    let r_a_r_x: & &u8 = &a.r_x;

    let r_a_x_ptr = asm(r: r_a_x) { r: raw_ptr };
    let r_a_r_x_ptr = asm(r: r_a_r_x) { r: raw_ptr };

    assert(r_a_x_ptr.read::<u8>() == x);
    assert(r_a_r_x_ptr.read::<raw_ptr>().read::<u8>() == x);

    assert(*r_a_x == x);
    assert(**r_a_r_x == x);

    let r_c_b_a_x: &u8 = &c.b.a.x;
    let r_c_b_a_r_x: & &u8 = &c.b.a.r_x;

    let r_c_b_a_x_ptr = asm(r: r_c_b_a_x) { r: raw_ptr };
    let r_c_b_a_r_x_ptr = asm(r: r_c_b_a_r_x) { r: raw_ptr };

    assert(r_c_b_a_x_ptr.read::<u8>() == x);
    assert(r_c_b_a_r_x_ptr.read::<raw_ptr>().read::<u8>() == x);

    assert(*r_c_b_a_x == x);
    assert(**r_c_b_a_r_x == x);

    assert(*c.r_b.r_a.r_x == x);
}

#[inline(never)]
fn struct_fields_not_inlined() {
    struct_fields()
}

#[inline(always)]
fn tuple_fields() {
    let x = 123u8;

    let t1 = (x, &x);
    let t2 = (t1, &t1);
    let t3 = (t2, &t2);

    let r_t1_x: &u8 = &t1.0;
    let r_t1_r_x: & &u8 = &t1.1;

    let r_t1_x_ptr = asm(r: r_t1_x) { r: raw_ptr };
    let r_t1_r_x_ptr = asm(r: r_t1_r_x) { r: raw_ptr };

    assert(r_t1_x_ptr.read::<u8>() == x);
    assert(r_t1_r_x_ptr.read::<raw_ptr>().read::<u8>() == x);

    assert(*r_t1_x == x);
    assert(**r_t1_r_x == x);

    let r_t3_t2_t1_x: &u8 = &t3.0.0.0;
    let r_t3_t2_t1_r_x: & &u8 = &t3.0.0.1;

    let r_t3_t2_t1_x_ptr = asm(r: r_t3_t2_t1_x) { r: raw_ptr };
    let r_t3_t2_t1_r_x_ptr = asm(r: r_t3_t2_t1_r_x) { r: raw_ptr };

    assert(r_t3_t2_t1_x_ptr.read::<u8>() == x);
    assert(r_t3_t2_t1_r_x_ptr.read::<raw_ptr>().read::<u8>() == x);

    assert(*r_t3_t2_t1_x == x);
    assert(**r_t3_t2_t1_r_x == x);
}

#[inline(never)]
fn tuple_fields_not_inlined() {
    tuple_fields()
}

#[inline(always)]
fn array_elements() {
    let x1 = 111u8;
    let x2 = 222u8;

    let a1 = [x1, x2];
    let a2 = [a1, a1];
    let a3 = [a2, a2];

    let r_a1_x1: &u8 = &a1[0];
    let r_a1_x2: &u8 = &a1[1];

    let r_a1_x1_ptr = asm(r: r_a1_x1) { r: raw_ptr };
    let r_a1_x2_ptr = asm(r: r_a1_x2) { r: raw_ptr };

    assert(r_a1_x1_ptr.read::<u8>() == x1);
    assert(r_a1_x2_ptr.read::<u8>() == x2);

    assert(*r_a1_x1 == x1);
    assert(*r_a1_x2 == x2);

    let r_a3_a2_a1_x1: &u8 = &a3[0][1][0];
    let r_a3_a2_a1_x2: &u8 = &a3[1][0][1];

    let r_a3_a2_a1_x1_ptr = asm(r: r_a3_a2_a1_x1) { r: raw_ptr };
    let r_a3_a2_a1_x2_ptr = asm(r: r_a3_a2_a1_x2) { r: raw_ptr };

    assert(r_a3_a2_a1_x1_ptr.read::<u8>() == x1);
    assert(r_a3_a2_a1_x2_ptr.read::<u8>() == x2);

    assert(*r_a3_a2_a1_x1 == x1);
    assert(*r_a3_a2_a1_x2 == x2);

    let a_r1 = [&x1, &x2];
    let a_r2 = [&a_r1, &a_r1];
    let a_r3 = [&a_r2, &a_r2];

    let r_a_r3_a_r2_a_r1_x1: & &u8 = &a_r3[0][1][0];
    assert(**r_a_r3_a_r2_a_r1_x1 == x1);

    assert(*(&a_r3)[0][0][0] == x1);
    assert(*(& &a_r3)[0][1][0] == x1);
    assert(*(& & &a_r3)[0][0][1] == x2);
    assert(*(& & & &a_r3)[0][1][1] == x2);

    assert(a3[0][1][0] == x1);
}

#[inline(never)]
fn array_elements_not_inlined() {
    array_elements()
}

struct S {
    a: [(u32, u32);2]
}

#[inline(always)]
fn all_in_one() {
    let s = S { a: [(222, 333), (444, 555)] };

    let r_222: &u32 = &s.a[0].0;
    let r_555: &u32 = &s.a[1].1;

    let r_222_ptr = asm(r: r_222) { r: raw_ptr };
    let r_555_ptr = asm(r: r_555) { r: raw_ptr };

    assert(r_222_ptr.read::<u32>() == 222);
    assert(r_555_ptr.read::<u32>() == 555);

    assert(*r_222 == 222);
    assert(*r_555 == 555);

    // ----

    let s1 = S { a: [(1222, 1333), (1444, 1555)] };
    let s2 = S { a: [(2222, 2333), (2444, 2555)] };

    let a = [(s, s1), (s1, s2), (s, s2)];

    let r_1555 = &a[1].0.a[1].1;
    let r_2333 = &a[2].1.a[0].1;

    let r_1555_ptr = asm(r: r_1555) { r: raw_ptr };
    let r_2333_ptr = asm(r: r_2333) { r: raw_ptr };

    assert(r_1555_ptr.read::<u32>() == 1555);
    assert(r_2333_ptr.read::<u32>() == 2333);

    assert(*r_1555 == 1555);
    assert(*r_2333 == 2333);

    // ----
    
    let t = ([s, s1], [s1, s2], [s, s2]);

    let r_1555 = &t.1[0].a[1].1;
    let r_2333 = &t.2[1].a[0].1;

    let r_1555_ptr = asm(r: r_1555) { r: raw_ptr };
    let r_2333_ptr = asm(r: r_2333) { r: raw_ptr };

    assert(r_1555_ptr.read::<u32>() == 1555);
    assert(r_2333_ptr.read::<u32>() == 2333);

    assert(*r_1555 == 1555);
    assert(*r_2333 == 2333);
}

#[inline(never)]
fn all_in_one_not_inlined() {
    all_in_one()
}

#[inline(never)]
fn test_all_inlined() {
    struct_fields();
    tuple_fields();
    array_elements();
    all_in_one();
}

#[inline(never)]
fn test_not_inlined() {
    struct_fields_not_inlined();
    tuple_fields_not_inlined();
    array_elements_not_inlined();
    all_in_one_not_inlined();
}

fn main() -> u64 {
    test_all_inlined();
    test_not_inlined();

    A::new().use_me();
    B::new().use_me();
    C::new().use_me();

    42
}

#[inline(never)]
fn poke<T>(_x: T) { }