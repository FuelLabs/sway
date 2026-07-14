script;

// A mostly-zeroed struct that embeds another struct, with a single non-zero leaf
// deep inside the nested struct (zero-ratio 7/8 = 0.875). The root struct is
// `mem_clear_val`ed exactly once, the nested `init_aggr` is removed, and only the
// single non-zero leaf is stored via a GEP into the nested position.

fn main() -> u64 {
    let s = Outer {
        x: 0,
        inner: Inner { a: 0, b: 0, c: 7, d: 0 },
        y: 0,
        z: 0,
    };
    s.inner.c
}

struct Inner {
    a: u64,
    b: u64,
    c: u64,
    d: u64,
}

struct Outer {
    x: u64,
    inner: Inner,
    y: u64,
    z: u64,
}

// ::check-ir::
// check: init_aggr

// ::check-ir-optimized::
// pass: lower-init-aggr

// check: $(outer_ptr=$VAL) = get_local __ptr { u64, { u64, u64, u64, u64 }, u64, u64 }, __struct_init_0
// check: $(inner_ptr=$VAL) = get_local __ptr { u64, u64, u64, u64 }, __struct_init_1

// The whole root struct is zero-cleared.
// check: mem_clear_val $outer_ptr

// Only the single non-zero leaf is stored, via a two-level GEP into the nested
// struct (field 1 of `Outer`, field 2 of `Inner`).
// check: $(leaf_ptr=$VAL) = get_elem_ptr $outer_ptr, __ptr u64, $VAL, $VAL
// check: $(c_7=$VAL) = const u64 7
// check: store $c_7 to $leaf_ptr

// nextln: $VAL = load $VAL

// The nested temporary is never zero-cleared (only the root is), and there must
// be no `init_aggr` left after lowering.
// not: mem_clear_val $inner_ptr
// not: init_aggr
