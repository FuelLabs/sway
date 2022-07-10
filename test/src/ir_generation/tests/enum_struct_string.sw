script;

struct S {
    n: str[17],
    v: u64,
}

struct A {
    s: S,
    a: u64,
    b: bool,
}

enum B {
    B: A,
}

fn main() -> u64 {
    let b_val = B::B(A { s: S { n: " an odd length", v: 20 }, a: 10, b: false });
    if let B::B(b) = b_val {
        b.a
    } else {
        0
    }
}

// check: local ptr { u64, ( { { string<17>, u64 }, u64, bool } ) } b_val

// check: get_ptr ptr { u64, ( { { string<17>, u64 }, u64, bool } ) } b_val, ptr { u64, ( { { string<17>, u64 }, u64, bool } ) }, 0

// check: $(b_val_ptr=$VAL) = get_ptr ptr { u64, ( { { string<17>, u64 }, u64, bool } ) } b_val, ptr { u64, ( { { string<17>, u64 }, u64, bool } ) }, 0
// check: $(b_val_tag=$VAL) = extract_value $b_val_ptr, { u64, ( { { string<17>, u64 }, u64, bool } ) }, 0
// check: $(zero=$VAL) = const u64 0
// check: $(tag_matches=$VAL) = call $(eq_fn=$ID)($b_val_tag, $zero)
// check: cbr $tag_matches

// check: fn $eq_fn(self $MD: u64, other $MD: u64) -> bool
