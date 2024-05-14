script;

struct S {
    x: u8,
}

#[inline(always)]
fn pass_references_to_asm_blocks() {
    let mut x = 123u8;

    let r_x = &x;
    let r_mut_x = &mut x;

    let r_x_ptr_1 = asm(r: &x) { r: raw_ptr };
    let r_x_ptr_2 = asm(r: r_x) { r: raw_ptr };

    assert(r_x_ptr_1 == r_x_ptr_2);

    let r_mut_x_ptr_1 = asm(r: &mut x) { r: raw_ptr };
    let r_mut_x_ptr_2 = asm(r: r_mut_x) { r: raw_ptr };

    assert(r_mut_x_ptr_1 == r_mut_x_ptr_2);

    let r_x_ptr_1_val = r_x_ptr_1.read::<u8>();
    let r_x_ptr_2_val = r_x_ptr_2.read::<u8>();

    assert(r_x_ptr_1_val == 123u8);
    assert(r_x_ptr_2_val == 123u8);

    let r_mut_x_ptr_1_val = r_mut_x_ptr_1.read::<u8>();
    let r_mut_x_ptr_2_val = r_mut_x_ptr_2.read::<u8>();

    assert(r_mut_x_ptr_1_val == 123u8);
    assert(r_mut_x_ptr_2_val == 123u8);

    let r_val_ptr_1 = asm(r: &123u8) { r: raw_ptr };
    let r_val_ptr_2 = asm(r: &(100u8 + 23u8)) { r: raw_ptr };
    let r_val_ptr_3 = asm(r: &return_123u8()) { r: raw_ptr };

    assert(r_x_ptr_1 != r_val_ptr_1);
    assert(r_val_ptr_1 != r_val_ptr_2);
    assert(r_val_ptr_2 != r_val_ptr_3);
    assert(r_val_ptr_3 != r_x_ptr_1);

    let r_val_ptr_1_val = r_val_ptr_1.read::<u8>();
    let r_val_ptr_2_val = r_val_ptr_2.read::<u8>();
    let r_val_ptr_3_val = r_val_ptr_3.read::<u8>();
    
    assert(r_val_ptr_1_val == 123u8);
    assert(r_val_ptr_2_val == 123u8);
    assert(r_val_ptr_3_val == 123u8);

    let r_val_ptr_1 = asm(r: &mut 123u8) { r: raw_ptr };
    let r_val_ptr_2 = asm(r: &mut (100u8 + 23u8)) { r: raw_ptr };
    let r_val_ptr_3 = asm(r: &mut return_123u8()) { r: raw_ptr };

    assert(r_x_ptr_1 != r_val_ptr_1);
    assert(r_val_ptr_1 != r_val_ptr_2);
    assert(r_val_ptr_2 != r_val_ptr_3);
    assert(r_val_ptr_3 != r_x_ptr_1);

    let r_val_ptr_1_val = r_val_ptr_1.read::<u8>();
    let r_val_ptr_2_val = r_val_ptr_2.read::<u8>();
    let r_val_ptr_3_val = r_val_ptr_3.read::<u8>();
    
    assert(r_val_ptr_1_val == 123u8);
    assert(r_val_ptr_2_val == 123u8);
    assert(r_val_ptr_3_val == 123u8);
}

#[inline(never)]
fn pass_references_to_asm_blocks_not_inlined() {
    pass_references_to_asm_blocks()
}

#[inline(never)]
fn return_123u8() -> u8 {
    123
}

#[inline(always)]
fn return_references_from_asm_blocks() {
    let x = 123u8;
    let r_x = &x;

    let r_x_ptr = asm(r: &x) { r: raw_ptr };

    let r_x_ref_1 = asm(r: &x) { r: &u8 };
    let r_x_ref_2 = asm(r: r_x) { r: &u8 };

    let r_x_ref_1_ptr = asm(r: r_x_ref_1) { r: raw_ptr };
    let r_x_ref_2_ptr = asm(r: r_x_ref_2) { r: raw_ptr };

    assert(r_x_ptr == r_x_ref_1_ptr);
    assert(r_x_ptr == r_x_ref_2_ptr);

    // Note that using asm we can circumvent mutability
    // checks and obtain a reference to mutable value that
    // refer to a non mutable variable.
    let r_x_ref_1: &mut u8 = asm(r: &x) { r: &mut u8 };
    let r_x_ref_2: &mut u8 = asm(r: r_x) { r: &mut u8 };

    let r_x_ref_1_ptr = asm(r: r_x_ref_1) { r: raw_ptr };
    let r_x_ref_2_ptr = asm(r: r_x_ref_2) { r: raw_ptr };

    assert(r_x_ptr == r_x_ref_1_ptr);
    assert(r_x_ptr == r_x_ref_2_ptr);

    // ----

    let s = S { x: 222 };
    let r_s = &s;

    let r_s_ptr = asm(r: &s) { r: raw_ptr };

    let r_s_ref_1 = asm(r: &s) { r: &S };
    let r_s_ref_2 = asm(r: r_s) { r: &S };

    let r_s_ref_1_ptr = asm(r: r_s_ref_1) { r: raw_ptr };
    let r_s_ref_2_ptr = asm(r: r_s_ref_2) { r: raw_ptr };

    assert(r_s_ptr == r_s_ref_1_ptr);
    assert(r_s_ptr == r_s_ref_2_ptr);

    let s_x = r_s_ref_1_ptr.read::<u8>();

    assert(s_x == s.x);

    assert((*r_s).x == s.x);
    assert((*r_s_ref_1).x == s.x);
    assert((*r_s_ref_2).x == s.x);

    // ----
    // Since aggregates are passed by reference we can always
    // cast a reference to an aggregate to the aggregate itself.
    // Note that the assignments below will make two copies,
    // as well ast the two `asm` blocks.
    let s_1 = asm(r: &s) { r: S };
    let s_2 = asm(r: r_s) { r: S };

    assert(s_1.x == s.x);
    assert(s_2.x == s.x);

    assert(asm(r: &s) { r: S }.x == s.x);
    assert(asm(r: r_s) { r: S }.x == s.x);
}

#[inline(never)]
fn return_references_from_asm_blocks_not_inlined() {
    return_references_from_asm_blocks()
}

#[inline(never)]
fn test_all_inlined() {
    pass_references_to_asm_blocks();
    return_references_from_asm_blocks();
}

#[inline(never)]
fn test_not_inlined() {
    pass_references_to_asm_blocks_not_inlined();
    return_references_from_asm_blocks_not_inlined();
}

fn main() -> u64 {
    test_all_inlined();
    test_not_inlined();

    42
}
