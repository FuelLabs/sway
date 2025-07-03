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
    let b_val = B::B(A { s: S { n: __to_str_array(" an odd length"), v: 20 }, a: 10, b: false });
    if let B::B(b) = b_val {
        b.a
    } else {
        0
    }
}

// check: local { u64, ( { { string<17>, u64 }, u64, bool } ) } b_val

// check: get_local __ptr { u64, ( { { string<17>, u64 }, u64, bool } ) }, b_val
// check: $(b_val_var=$VAL) = get_local __ptr { u64, ( { { string<17>, u64 }, u64, bool } ) }, b_val

// check: get_local __ptr { u64, ( { { string<17>, u64 }, u64, bool } ) }, __matched_value_1
// check: $(match_val_var=$VAL) = get_local __ptr { u64, ( { { string<17>, u64 }, u64, bool } ) }, __matched_value_1

// check: $(idx_val=$VAL) = const u64 0
// check: $(tag_ptr=$VAL) = get_elem_ptr $match_val_var, __ptr u64, $idx_val
// check: $(b_val_tag=$VAL) = load $tag_ptr
// check: $(zero=$VAL) = const u64 0
// check: $(tag_matches=$VAL) = call $(eq_fn=$ID)($b_val_tag, $zero)
// check: cbr $tag_matches

// check: fn $eq_fn(self $MD: u64, other $MD: u64) -> bool
