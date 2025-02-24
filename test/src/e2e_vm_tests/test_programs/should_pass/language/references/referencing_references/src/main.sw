script;

#[inline(always)]
fn referencing_references() {
    let x = 123u8;

    let r_x = &x;

    let r_x_1 = &x;
    let r_x_2 = &x;

    let r_r_x_1 = &r_x_1;
    let r_r_x_2 = &r_x_2;

    let r_r_x = &r_x;
    let r_r_r_x = &r_r_x;

    // TODO: (REFERENCES) Remove space once parsing supports `&&&..&&x`.
    let r_r_x_chain = & &x;
    let r_r_r_x_chain = & & &x;

    let r_x_ptr = asm(r: r_x) { r: raw_ptr };
    let r_x_1_ptr = asm(r: r_x_1) { r: raw_ptr };
    let r_x_2_ptr = asm(r: r_x_2) { r: raw_ptr };

    assert(r_x_ptr == r_x_1_ptr);
    assert(r_x_ptr == r_x_2_ptr);

    let r_r_x_1_ptr = asm(r: r_r_x_1) { r: raw_ptr };
    let r_r_x_2_ptr = asm(r: r_r_x_2) { r: raw_ptr };
    
    let r_r_x_ptr = asm(r: r_r_x) { r: raw_ptr };
    let r_r_r_x_ptr = asm(r: r_r_r_x) { r: raw_ptr };
    
    let r_r_x_chain_ptr = asm(r: r_r_x_chain) { r: raw_ptr };
    let r_r_r_x_chain_ptr = asm(r: r_r_r_x_chain) { r: raw_ptr };

    assert(r_x_ptr != r_r_x_ptr);
    assert(r_r_x_ptr != r_r_r_x_ptr);
    assert(r_r_r_x_ptr != r_x_ptr);
    
    // `r_x_1` and `r_x_2` references the same variable `x`
    // but they are two different variables. That means that
    // the references pointing to them must be different.
    // Same is with the `chain` references and their "`chain`-less"
    // counterparts.
    assert(r_r_x_1_ptr != r_r_x_2_ptr);

    assert(r_r_x_ptr != r_r_x_chain_ptr);
    assert(r_r_r_x_ptr != r_r_r_x_chain_ptr);

    let x_via_refs = r_r_r_x_ptr.read::<raw_ptr>().read::<raw_ptr>().read::<u8>();
    assert(x_via_refs == x);

    assert(*r_x == x);
    assert(*r_x_1 == x);
    assert(*r_x_2 == x);
    assert(**r_r_x_1 == x);
    assert(**r_r_x_2 == x);
    assert(**r_r_x_chain == x);
    assert(***r_r_r_x_chain == x);
}

#[inline(never)]
fn referencing_references_not_inlined() {
    referencing_references()
}

#[inline(never)]
fn test_all_inlined() {
    referencing_references();
}

#[inline(never)]
fn test_not_inlined() {
    referencing_references_not_inlined();
}

fn main() -> u64 {
    test_all_inlined();
    test_not_inlined();

    42
}
