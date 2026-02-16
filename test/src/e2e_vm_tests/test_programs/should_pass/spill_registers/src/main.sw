// This test proves that https://github.com/FuelLabs/sway/issues/7547 is fixed.
script;

#[inline(never)]
fn dummy() -> u64 {
    42
}

#[inline(never)]
fn reg_pressure() -> u64 {
    let v01: u64 = dummy();
    let v02: u64 = dummy();
    let v03: u64 = dummy();
    let v04: u64 = dummy();
    let v05: u64 = dummy();
    let v06: u64 = dummy();
    let v07: u64 = dummy();
    let v08: u64 = dummy();
    let v09: u64 = dummy();
    let v10: u64 = dummy();
    let v11: u64 = dummy();
    let v12: u64 = dummy();
    let v13: u64 = dummy();
    let v14: u64 = dummy();
    let v15: u64 = dummy();
    let v16: u64 = dummy();
    let v17: u64 = dummy();
    let v18: u64 = dummy();
    let v19: u64 = dummy();
    let v20: u64 = dummy();
    let v21: u64 = dummy();
    let v22: u64 = dummy();
    let v23: u64 = dummy();
    let v24: u64 = dummy();
    let v25: u64 = dummy();
    let v26: u64 = dummy();
    let v27: u64 = dummy();
    let v28: u64 = dummy();
    let v29: u64 = dummy();
    let v30: u64 = dummy();
    let v31: u64 = dummy();
    let v32: u64 = dummy();
    let v33: u64 = dummy();
    let v34: u64 = dummy();
    let v35: u64 = dummy();
    let v36: u64 = dummy();
    let v37: u64 = dummy();
    let v38: u64 = dummy();
    let v39: u64 = dummy();
    let v40: u64 = dummy();
    let v41: u64 = dummy();
    let v42: u64 = dummy();
    let v43: u64 = dummy();
    let v44: u64 = dummy();
    let v45: u64 = dummy();
    let v46: u64 = dummy();
    let v47: u64 = dummy();
    let v48: u64 = dummy();
    let v49: u64 = dummy();
    let v50: u64 = dummy();
    let v51: u64 = dummy();
    let v52: u64 = dummy();
    let v53: u64 = dummy();
    let v54: u64 = dummy();
    let v55: u64 = dummy();
    let v56: u64 = dummy();
    let v57: u64 = dummy();
    let v58: u64 = dummy();
    let v59: u64 = dummy();
    let v60: u64 = dummy();
    let v61: u64 = dummy();
    let v62: u64 = dummy();
    let v63: u64 = dummy();
    let v64: u64 = dummy();

    v01 + v02 + v03 + v04 + v05 + v06 + v07 + v08 + v09 + v10 + v11 + v12
        + v13 + v14 + v15 + v16 + v17 + v18 + v19 + v20 + v21 + v22 + v23
        + v24 + v25 + v26 + v27 + v28 + v29 + v30 + v31 + v32 + v33 + v34
        + v35 + v36 + v37 + v38 + v39 + v40 + v41 + v42 + v43 + v44 + v45
        + v46 + v47 + v48 + v49 + v50 + v51 + v52 + v53 + v54 + v55 + v56
        + v57 + v58 + v59 + v60 + v61 + v62 + v63 + v64
}

fn main(a: u64) -> u64 {
    reg_pressure()
}
