script;

struct A {
    a: u64,
}

fn main() -> u64 {
    let a = true;
    let a = if a { 12 } else { 21 };
    let a = A { a: a };
    a.a
}

// check: local ptr bool a
// check: local ptr u64 a_
// check: local ptr { u64 } a__

// check: $(a_ptr=$VAL) = get_ptr ptr bool a, ptr bool, 0
// check: $(true=$VAL) = const bool true
// check: store $true, ptr $a_ptr

// check: $ID($(int_val=$VAL):
// check: $(a__ptr=$VAL) = get_ptr ptr u64 a_, ptr u64, 0
// check: store $int_val, ptr $a__ptr

// check: $(struct_undef=$VAL) = get_ptr ptr { u64 } $ID, ptr { u64 }, 0
// check: $(struct_set=$VAL) = insert_value $struct_undef, { u64 }, v9, 0
// check: $(a___ptr=$VAL) = get_ptr ptr { u64 } a__, ptr { u64 }, 0
// check: store $struct_set, ptr $a___ptr
