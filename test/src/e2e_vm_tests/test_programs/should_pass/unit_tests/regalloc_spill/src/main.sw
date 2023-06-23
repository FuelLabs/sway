script;

fn main() -> u64 {
   fib_add()
}

fn fib_add() -> u64 {
    let v00 = 1;
    let v01 = 2;
    let v02 = add(v00, v01);
    let v03 = add(v01, v02);
    let v04 = add(v02, v03);
    let v05 = add(v03, v04);
    let v06 = add(v04, v05);
    let v07 = add(v05, v06);
    let v08 = add(v06, v07);
    let v09 = add(v07, v08);
    let v10 = add(v08, v09);
    let v11 = add(v09, v10);
    let v12 = add(v10, v11);
    let v13 = add(v11, v12);
    let v14 = add(v12, v13);
    let v15 = add(v13, v14);
    let v16 = add(v14, v15);
    let v17 = add(v15, v16);
    let v18 = add(v16, v17);
    let v19 = add(v17, v18);
    let v20 = add(v18, v19);
    let v21 = add(v19, v20);
    let v22 = add(v20, v21);
    let v23 = add(v21, v22);
    let v24 = add(v22, v23);
    let v25 = add(v23, v24);
    let v26 = add(v24, v25);
    let v27 = add(v25, v26);
    let v28 = add(v26, v27);
    let v29 = add(v27, v28);
    let v30 = add(v28, v29);
    let v31 = add(v29, v30);
    let v32 = add(v30, v31);
    let v33 = add(v31, v32);
    let v34 = add(v32, v33);
    let v35 = add(v33, v34);
    let v36 = add(v34, v35);
    let v37 = add(v35, v36);
    let v38 = add(v36, v37);
    let v39 = add(v37, v38);

    let res = if t() {
        add(add(add(add(add(v00, v01), v02), add(v03, v04)), add(add(add(v05, v06), v07), add(v08, v09))),
            add(add(add(add(v10, v11), v12), add(v13, v14)), add(add(add(v15, v16), v17), add(v18, v19))))
    } else {
        add(add(add(add(add(v20, v21), v22), add(v23, v24)), add(add(add(v25, v26), v27), add(v28, v29))),
            add(add(add(add(v30, v31), v32), add(v33, v34)), add(add(add(v35, v36), v37), add(v38, v39))))
    };

    res
}

fn add(l: u64, r: u64) -> u64 {
    asm(l: l, r: r, n) {
        add n l r;
        n: u64
    }
}

fn t() -> bool {
    asm() {
        one: bool
    }
}
