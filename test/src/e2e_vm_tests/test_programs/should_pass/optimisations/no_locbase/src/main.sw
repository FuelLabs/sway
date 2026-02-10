script;

#[inline(never)]
fn dummy(a: u64) -> u64 {
    a
}

#[inline(never)]
fn need_call_args_on_stack(a: u64, b: u64, c: u64, d: u64, e: u64, f: u64, g: u64, h: u64) -> u64 {
    a + b + c + d + e + f + g + h
}

fn main(a: u64) { }

struct S {
    a: u64,
    b: u64,
    c: u64,
    d: u64,
    e: u64,
    f: u64,
    g: u64,
    h: u64,
}

#[test]
fn test() {
    assert_eq(check_no_locals_no_call_args_no_spills(42), 42);
    assert_eq(check_no_locals_no_call_args_spills(42), 42*64);
    assert_eq(check_no_locals_call_args_no_spills(42), 42*8);
    assert_eq(check_no_locals_call_args_spills(42), 42*8 + 42*64);
    assert_eq(check_locals_no_call_args_no_spills(42), 42*8);
    // TODO: Enable this test once https://github.com/FuelLabs/sway/issues/7551 is fixed.
    // assert_eq(check_locals_no_call_args_spills(42), 42*8 + 42*64);
    assert_eq(check_locals_call_args_no_spills(42), 42*8 + 42*8);
    assert_eq(check_locals_call_args_spills(42), 42*8 + 42*8 + 42*64);
}

#[inline(never)]
fn check_no_locals_no_call_args_no_spills(a: u64) -> u64 {
    a
}

#[inline(never)]
fn check_no_locals_no_call_args_spills(a: u64) -> u64 {
    let v01: u64 = dummy(a);
    let v02: u64 = dummy(a);
    let v03: u64 = dummy(a);
    let v04: u64 = dummy(a);
    let v05: u64 = dummy(a);
    let v06: u64 = dummy(a);
    let v07: u64 = dummy(a);
    let v08: u64 = dummy(a);
    let v09: u64 = dummy(a);
    let v10: u64 = dummy(a);
    let v11: u64 = dummy(a);
    let v12: u64 = dummy(a);
    let v13: u64 = dummy(a);
    let v14: u64 = dummy(a);
    let v15: u64 = dummy(a);
    let v16: u64 = dummy(a);
    let v17: u64 = dummy(a);
    let v18: u64 = dummy(a);
    let v19: u64 = dummy(a);
    let v20: u64 = dummy(a);
    let v21: u64 = dummy(a);
    let v22: u64 = dummy(a);
    let v23: u64 = dummy(a);
    let v24: u64 = dummy(a);
    let v25: u64 = dummy(a);
    let v26: u64 = dummy(a);
    let v27: u64 = dummy(a);
    let v28: u64 = dummy(a);
    let v29: u64 = dummy(a);
    let v30: u64 = dummy(a);
    let v31: u64 = dummy(a);
    let v32: u64 = dummy(a);
    let v33: u64 = dummy(a);
    let v34: u64 = dummy(a);
    let v35: u64 = dummy(a);
    let v36: u64 = dummy(a);
    let v37: u64 = dummy(a);
    let v38: u64 = dummy(a);
    let v39: u64 = dummy(a);
    let v40: u64 = dummy(a);
    let v41: u64 = dummy(a);
    let v42: u64 = dummy(a);
    let v43: u64 = dummy(a);
    let v44: u64 = dummy(a);
    let v45: u64 = dummy(a);
    let v46: u64 = dummy(a);
    let v47: u64 = dummy(a);
    let v48: u64 = dummy(a);
    let v49: u64 = dummy(a);
    let v50: u64 = dummy(a);
    let v51: u64 = dummy(a);
    let v52: u64 = dummy(a);
    let v53: u64 = dummy(a);
    let v54: u64 = dummy(a);
    let v55: u64 = dummy(a);
    let v56: u64 = dummy(a);
    let v57: u64 = dummy(a);
    let v58: u64 = dummy(a);
    let v59: u64 = dummy(a);
    let v60: u64 = dummy(a);
    let v61: u64 = dummy(a);
    let v62: u64 = dummy(a);
    let v63: u64 = dummy(a);
    let v64: u64 = dummy(a);

    v01 + v02 + v03 + v04 + v05 + v06 + v07 + v08 + v09 + v10 + v11 + v12
        + v13 + v14 + v15 + v16 + v17 + v18 + v19 + v20 + v21 + v22 + v23
        + v24 + v25 + v26 + v27 + v28 + v29 + v30 + v31 + v32 + v33 + v34
        + v35 + v36 + v37 + v38 + v39 + v40 + v41 + v42 + v43 + v44 + v45
        + v46 + v47 + v48 + v49 + v50 + v51 + v52 + v53 + v54 + v55 + v56
        + v57 + v58 + v59 + v60 + v61 + v62 + v63 + v64
}

#[inline(never)]
fn check_no_locals_call_args_no_spills(a: u64) -> u64 {
    need_call_args_on_stack(a, a, a, a, a, a, a, a)
}

#[inline(never)]
fn check_no_locals_call_args_spills(a: u64) -> u64 {
    let v = need_call_args_on_stack(a, a, a, a, a, a, a, a);

    let v01: u64 = dummy(a);
    let v02: u64 = dummy(a);
    let v03: u64 = dummy(a);
    let v04: u64 = dummy(a);
    let v05: u64 = dummy(a);
    let v06: u64 = dummy(a);
    let v07: u64 = dummy(a);
    let v08: u64 = dummy(a);
    let v09: u64 = dummy(a);
    let v10: u64 = dummy(a);
    let v11: u64 = dummy(a);
    let v12: u64 = dummy(a);
    let v13: u64 = dummy(a);
    let v14: u64 = dummy(a);
    let v15: u64 = dummy(a);
    let v16: u64 = dummy(a);
    let v17: u64 = dummy(a);
    let v18: u64 = dummy(a);
    let v19: u64 = dummy(a);
    let v20: u64 = dummy(a);
    let v21: u64 = dummy(a);
    let v22: u64 = dummy(a);
    let v23: u64 = dummy(a);
    let v24: u64 = dummy(a);
    let v25: u64 = dummy(a);
    let v26: u64 = dummy(a);
    let v27: u64 = dummy(a);
    let v28: u64 = dummy(a);
    let v29: u64 = dummy(a);
    let v30: u64 = dummy(a);
    let v31: u64 = dummy(a);
    let v32: u64 = dummy(a);
    let v33: u64 = dummy(a);
    let v34: u64 = dummy(a);
    let v35: u64 = dummy(a);
    let v36: u64 = dummy(a);
    let v37: u64 = dummy(a);
    let v38: u64 = dummy(a);
    let v39: u64 = dummy(a);
    let v40: u64 = dummy(a);
    let v41: u64 = dummy(a);
    let v42: u64 = dummy(a);
    let v43: u64 = dummy(a);
    let v44: u64 = dummy(a);
    let v45: u64 = dummy(a);
    let v46: u64 = dummy(a);
    let v47: u64 = dummy(a);
    let v48: u64 = dummy(a);
    let v49: u64 = dummy(a);
    let v50: u64 = dummy(a);
    let v51: u64 = dummy(a);
    let v52: u64 = dummy(a);
    let v53: u64 = dummy(a);
    let v54: u64 = dummy(a);
    let v55: u64 = dummy(a);
    let v56: u64 = dummy(a);
    let v57: u64 = dummy(a);
    let v58: u64 = dummy(a);
    let v59: u64 = dummy(a);
    let v60: u64 = dummy(a);
    let v61: u64 = dummy(a);
    let v62: u64 = dummy(a);
    let v63: u64 = dummy(a);
    let v64: u64 = dummy(a);

    v + v01 + v02 + v03 + v04 + v05 + v06 + v07 + v08 + v09 + v10 + v11 + v12
        + v13 + v14 + v15 + v16 + v17 + v18 + v19 + v20 + v21 + v22 + v23
        + v24 + v25 + v26 + v27 + v28 + v29 + v30 + v31 + v32 + v33 + v34
        + v35 + v36 + v37 + v38 + v39 + v40 + v41 + v42 + v43 + v44 + v45
        + v46 + v47 + v48 + v49 + v50 + v51 + v52 + v53 + v54 + v55 + v56
        + v57 + v58 + v59 + v60 + v61 + v62 + v63 + v64
}

#[inline(never)]
fn check_locals_no_call_args_no_spills(a: u64) -> u64 {
    let s = S {
        a,
        b: a,
        c: a,
        d: a,
        e: a,
        f: a,
        g: a,
        h: a,
    };

    s.a + s.b + s.c + s.d + s.e + s.f + s.g + s.h
}

// #[inline(never)]
// fn check_locals_no_call_args_spills(a: u64) -> u64 {
//     let s = S {
//         a,
//         b: a,
//         c: a,
//         d: a,
//         e: a,
//         f: a,
//         g: a,
//         h: a,
//     };

//     let v01: u64 = dummy(a);
//     let v02: u64 = dummy(a);
//     let v03: u64 = dummy(a);
//     let v04: u64 = dummy(a);
//     let v05: u64 = dummy(a);
//     let v06: u64 = dummy(a);
//     let v07: u64 = dummy(a);
//     let v08: u64 = dummy(a);
//     let v09: u64 = dummy(a);
//     let v10: u64 = dummy(a);
//     let v11: u64 = dummy(a);
//     let v12: u64 = dummy(a);
//     let v13: u64 = dummy(a);
//     let v14: u64 = dummy(a);
//     let v15: u64 = dummy(a);
//     let v16: u64 = dummy(a);
//     let v17: u64 = dummy(a);
//     let v18: u64 = dummy(a);
//     let v19: u64 = dummy(a);
//     let v20: u64 = dummy(a);
//     let v21: u64 = dummy(a);
//     let v22: u64 = dummy(a);
//     let v23: u64 = dummy(a);
//     let v24: u64 = dummy(a);
//     let v25: u64 = dummy(a);
//     let v26: u64 = dummy(a);
//     let v27: u64 = dummy(a);
//     let v28: u64 = dummy(a);
//     let v29: u64 = dummy(a);
//     let v30: u64 = dummy(a);
//     let v31: u64 = dummy(a);
//     let v32: u64 = dummy(a);
//     let v33: u64 = dummy(a);
//     let v34: u64 = dummy(a);
//     let v35: u64 = dummy(a);
//     let v36: u64 = dummy(a);
//     let v37: u64 = dummy(a);
//     let v38: u64 = dummy(a);
//     let v39: u64 = dummy(a);
//     let v40: u64 = dummy(a);
//     let v41: u64 = dummy(a);
//     let v42: u64 = dummy(a);
//     let v43: u64 = dummy(a);
//     let v44: u64 = dummy(a);
//     let v45: u64 = dummy(a);
//     let v46: u64 = dummy(a);
//     let v47: u64 = dummy(a);
//     let v48: u64 = dummy(a);
//     let v49: u64 = dummy(a);
//     let v50: u64 = dummy(a);
//     let v51: u64 = dummy(a);
//     let v52: u64 = dummy(a);
//     let v53: u64 = dummy(a);
//     let v54: u64 = dummy(a);
//     let v55: u64 = dummy(a);
//     let v56: u64 = dummy(a);
//     let v57: u64 = dummy(a);
//     let v58: u64 = dummy(a);
//     let v59: u64 = dummy(a);
//     let v60: u64 = dummy(a);
//     let v61: u64 = dummy(a);
//     let v62: u64 = dummy(a);
//     let v63: u64 = dummy(a);
//     let v64: u64 = dummy(a);

//     v + v01 + v02 + v03 + v04 + v05 + v06 + v07 + v08 + v09 + v10 + v11 + v12
//         + v13 + v14 + v15 + v16 + v17 + v18 + v19 + v20 + v21 + v22 + v23
//         + v24 + v25 + v26 + v27 + v28 + v29 + v30 + v31 + v32 + v33 + v34
//         + v35 + v36 + v37 + v38 + v39 + v40 + v41 + v42 + v43 + v44 + v45
//         + v46 + v47 + v48 + v49 + v50 + v51 + v52 + v53 + v54 + v55 + v56
//         + v57 + v58 + v59 + v60 + v61 + v62 + v63 + v64
//         + s.a + s.b + s.c + s.d + s.e + s.f + s.g + s.h
// }

#[inline(never)]
fn check_locals_call_args_no_spills(a: u64) -> u64 {
    let v = need_call_args_on_stack(a, a, a, a, a, a, a, a);

    let s = S {
        a,
        b: a,
        c: a,
        d: a,
        e: a,
        f: a,
        g: a,
        h: a,
    };

    v + s.a + s.b + s.c + s.d + s.e + s.f + s.g + s.h
}

#[inline(never)]
fn check_locals_call_args_spills(a: u64) -> u64 {
    let v = need_call_args_on_stack(a, a, a, a, a, a, a, a);

    let s = S {
        a,
        b: a,
        c: a,
        d: a,
        e: a,
        f: a,
        g: a,
        h: a,
    };

    let v01: u64 = dummy(a);
    let v02: u64 = dummy(a);
    let v03: u64 = dummy(a);
    let v04: u64 = dummy(a);
    let v05: u64 = dummy(a);
    let v06: u64 = dummy(a);
    let v07: u64 = dummy(a);
    let v08: u64 = dummy(a);
    let v09: u64 = dummy(a);
    let v10: u64 = dummy(a);
    let v11: u64 = dummy(a);
    let v12: u64 = dummy(a);
    let v13: u64 = dummy(a);
    let v14: u64 = dummy(a);
    let v15: u64 = dummy(a);
    let v16: u64 = dummy(a);
    let v17: u64 = dummy(a);
    let v18: u64 = dummy(a);
    let v19: u64 = dummy(a);
    let v20: u64 = dummy(a);
    let v21: u64 = dummy(a);
    let v22: u64 = dummy(a);
    let v23: u64 = dummy(a);
    let v24: u64 = dummy(a);
    let v25: u64 = dummy(a);
    let v26: u64 = dummy(a);
    let v27: u64 = dummy(a);
    let v28: u64 = dummy(a);
    let v29: u64 = dummy(a);
    let v30: u64 = dummy(a);
    let v31: u64 = dummy(a);
    let v32: u64 = dummy(a);
    let v33: u64 = dummy(a);
    let v34: u64 = dummy(a);
    let v35: u64 = dummy(a);
    let v36: u64 = dummy(a);
    let v37: u64 = dummy(a);
    let v38: u64 = dummy(a);
    let v39: u64 = dummy(a);
    let v40: u64 = dummy(a);
    let v41: u64 = dummy(a);
    let v42: u64 = dummy(a);
    let v43: u64 = dummy(a);
    let v44: u64 = dummy(a);
    let v45: u64 = dummy(a);
    let v46: u64 = dummy(a);
    let v47: u64 = dummy(a);
    let v48: u64 = dummy(a);
    let v49: u64 = dummy(a);
    let v50: u64 = dummy(a);
    let v51: u64 = dummy(a);
    let v52: u64 = dummy(a);
    let v53: u64 = dummy(a);
    let v54: u64 = dummy(a);
    let v55: u64 = dummy(a);
    let v56: u64 = dummy(a);
    let v57: u64 = dummy(a);
    let v58: u64 = dummy(a);
    let v59: u64 = dummy(a);
    let v60: u64 = dummy(a);
    let v61: u64 = dummy(a);
    let v62: u64 = dummy(a);
    let v63: u64 = dummy(a);
    let v64: u64 = dummy(a);

    v + s.a + s.b + s.c + s.d + s.e + s.f + s.g + s.h
        + v01 + v02 + v03 + v04 + v05 + v06 + v07 + v08 + v09 + v10 + v11 + v12
        + v13 + v14 + v15 + v16 + v17 + v18 + v19 + v20 + v21 + v22 + v23
        + v24 + v25 + v26 + v27 + v28 + v29 + v30 + v31 + v32 + v33 + v34
        + v35 + v36 + v37 + v38 + v39 + v40 + v41 + v42 + v43 + v44 + v45
        + v46 + v47 + v48 + v49 + v50 + v51 + v52 + v53 + v54 + v55 + v56
        + v57 + v58 + v59 + v60 + v61 + v62 + v63 + v64

}