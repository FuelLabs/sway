script;

#[inline(always)]
fn bool() {
    let x = true;

    let r_x_1 = &x;
    let r_x_2 = &x;
    let r_val = &true;

    let r_x_1_ptr = asm(r: r_x_1) { r: raw_ptr };
    let r_x_2_ptr = asm(r: r_x_2) { r: raw_ptr };
    let r_val_ptr = asm(r: r_val) { r: raw_ptr };

    assert(r_x_1_ptr == r_x_2_ptr);
    assert(r_x_1_ptr != r_val_ptr);

    let r_x_1_ptr_val = r_x_1_ptr.read::<bool>();
    let r_x_2_ptr_val = r_x_2_ptr.read::<bool>();
    let r_x_val_val = r_val_ptr.read::<bool>();

    assert(r_x_1_ptr_val == x);
    assert(r_x_2_ptr_val == x);
    assert(r_x_val_val == true);
}

#[inline(never)]
fn bool_not_inlined() {
    bool()
}

#[inline(always)]
fn unsigned_u8() {
    let x = 123u8;

    let r_x_1 = &x;
    let r_x_2 = &x;
    let r_val = &123u8;

    let r_x_1_ptr = asm(r: r_x_1) { r: raw_ptr };
    let r_x_2_ptr = asm(r: r_x_2) { r: raw_ptr };
    let r_val_ptr = asm(r: r_val) { r: raw_ptr };

    assert(r_x_1_ptr == r_x_2_ptr);
    assert(r_x_1_ptr != r_val_ptr);

    let r_x_1_ptr_val = r_x_1_ptr.read::<u8>();
    let r_x_2_ptr_val = r_x_2_ptr.read::<u8>();
    let r_x_val_val = r_val_ptr.read::<u8>();

    assert(r_x_1_ptr_val == x);
    assert(r_x_2_ptr_val == x);
    assert(r_x_val_val == 123u8);
}

#[inline(never)]
fn unsigned_u8_not_inlined() {
    unsigned_u8()
}

#[inline(always)]
fn unsigned_u32() {
    let x = 123u32;

    let r_x_1 = &x;
    let r_x_2 = &x;
    let r_val = &123u32;

    let r_x_1_ptr = asm(r: r_x_1) { r: raw_ptr };
    let r_x_2_ptr = asm(r: r_x_2) { r: raw_ptr };
    let r_val_ptr = asm(r: r_val) { r: raw_ptr };

    assert(r_x_1_ptr == r_x_2_ptr);
    assert(r_x_1_ptr != r_val_ptr);

    let r_x_1_ptr_val = r_x_1_ptr.read::<u32>();
    let r_x_2_ptr_val = r_x_2_ptr.read::<u32>();
    let r_x_val_val = r_val_ptr.read::<u32>();

    assert(r_x_1_ptr_val == x);
    assert(r_x_2_ptr_val == x);
    assert(r_x_val_val == 123u32);
}

#[inline(never)]
fn unsigned_u32_not_inlined() {
    unsigned_u32()
}

#[inline(always)]
fn array() {
    let x = [123u32];

    let r_x_1 = &x;
    let r_x_2 = &x;
    let r_val = &[123u32];

    let r_x_1_ptr = asm(r: r_x_1) { r: raw_ptr };
    let r_x_2_ptr = asm(r: r_x_2) { r: raw_ptr };
    let r_val_ptr = asm(r: r_val) { r: raw_ptr };

    assert(r_x_1_ptr == r_x_2_ptr);
    assert(r_x_1_ptr != r_val_ptr);

    let r_x_1_ptr_val = r_x_1_ptr.read::<[u32;1]>();
    let r_x_2_ptr_val = r_x_2_ptr.read::<[u32;1]>();
    let r_x_val_val = r_val_ptr.read::<[u32;1]>();

    assert(r_x_1_ptr_val[0] == x[0]);
    assert(r_x_2_ptr_val[0] == x[0]);
    assert(r_x_val_val[0] == 123u32);
}

#[inline(never)]
fn array_not_inlined() {
    array()
}

struct EmptyStruct { }

impl core::ops::Eq for EmptyStruct {
    fn eq(self, other: Self) -> bool {
        true
    }
}

struct Struct {
    x: u64,
}

impl core::ops::Eq for Struct {
    fn eq(self, other: Self) -> bool {
        self.x == other.x
    }
}

#[inline(always)]
fn empty_struct(is_inlined: bool) {
    let x = EmptyStruct { };

    let r_x_1 = &x;
    let r_x_2 = &x;
    let r_val = &EmptyStruct { };

    let r_dummy = &123u64;

    let r_x_1_ptr = asm(r: r_x_1) { r: raw_ptr };
    let r_x_2_ptr = asm(r: r_x_2) { r: raw_ptr };
    let r_val_ptr = asm(r: r_val) { r: raw_ptr };
    let r_dummy_ptr = asm(r: r_dummy) { r: raw_ptr };

    // If there is no inlining and mixing with other test functions,
    // since the struct is empty, means allocates zero memory, both structs
    // will be on the same memory location.
    // The dummy value will also be on the same location.
    // In case of inlining with other test functions we can get
    // the two structs position separately from each other intermixed
    // with the locals coming from other functions.
    if (!is_inlined) {
        assert(r_x_1_ptr == r_val_ptr);
        assert(r_x_1_ptr == r_dummy_ptr);
    }

    assert(r_x_1_ptr == r_x_2_ptr);

    let r_x_1_ptr_val = r_x_1_ptr.read::<EmptyStruct>();
    let r_x_2_ptr_val = r_x_2_ptr.read::<EmptyStruct>();
    let r_x_val_val = r_val_ptr.read::<EmptyStruct>();
    let r_dummy_val = r_dummy_ptr.read::<u64>();

    assert(r_x_1_ptr_val == x);
    assert(r_x_2_ptr_val == x);
    assert(r_x_val_val == EmptyStruct { });
    assert(r_dummy_val == 123);
}

#[inline(never)]
fn empty_struct_not_inlined() {
    empty_struct(false)
}

#[inline(always)]
fn non_empty_struct() {
    let x = Struct { x: 123 };

    let r_x_1 = &x;
    let r_x_2 = &x;
    let r_val = &Struct { x: 321 };

    let r_x_1_ptr = asm(r: r_x_1) { r: raw_ptr };
    let r_x_2_ptr = asm(r: r_x_2) { r: raw_ptr };
    let r_val_ptr = asm(r: r_val) { r: raw_ptr };

    assert(r_x_1_ptr == r_x_2_ptr);
    assert(r_x_1_ptr != r_val_ptr);

    let r_x_1_ptr_val = r_x_1_ptr.read::<Struct>();
    let r_x_2_ptr_val = r_x_2_ptr.read::<Struct>();
    let r_x_val_val = r_val_ptr.read::<Struct>();

    assert(r_x_1_ptr_val == x);
    assert(r_x_2_ptr_val == x);
    assert(r_x_val_val == Struct { x: 321 });
}

#[inline(never)]
fn non_empty_struct_not_inlined() {
    non_empty_struct()
}

#[inline(never)]
fn test_all_inlined() {
    bool();
    unsigned_u8();
    unsigned_u32();
    array();
    empty_struct(true);
    non_empty_struct();
}

#[inline(never)]
fn test_not_inlined() {
    bool_not_inlined();
    unsigned_u8_not_inlined();
    unsigned_u32_not_inlined();
    array_not_inlined();
    empty_struct_not_inlined();
    non_empty_struct_not_inlined();
}

fn main() -> u64 {
    test_all_inlined();
    test_not_inlined();

    42
}
