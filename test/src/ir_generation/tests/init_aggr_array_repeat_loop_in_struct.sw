script;

// A large repeat array (length > 5) nested inside a struct. The array is
// lowered into an initialization loop, but unlike the top-level case, the loop
// operates on a pointer computed with a `get_elem_ptr` into the struct (the
// non-empty GEP-indices branch of the lowering). The remaining scalar field is
// stored in the loop's exit block.

struct S {
    head: u64,
    body: [u64; 10],
}

fn main() -> u64 {
    let s = S {
        head: 3,
        body: [7u64; 10],
    };
    s.body[1]
}

// ::check-ir::

// check: local { u64, [u64; 10] } __struct_init_0

// check: $(ptr_struct_init=$VAL) = get_local __ptr { u64, [u64; 10] }, __struct_init_0
// check: init_aggr

// ::check-ir-optimized::
// pass: lower-init-aggr

// check: $(ptr_struct_init=$VAL) = get_local __ptr { u64, [u64; 10] }, __struct_init_0

// The pointer to the nested array field is computed with a GEP into the struct.
// check: get_elem_ptr $ptr_struct_init, __ptr [u64; 10]

// The array is initialized with a loop.
// check: br array_init_loop($VAL)

// check: array_init_loop(mut $(index=$VAL): u64):
// check: $(ptr_elem=$VAL) = get_elem_ptr $(body_ptr=$VAL), __ptr u64, $index
// check: $(c_7=$VAL) = const u64 7
// check: store $c_7 to $ptr_elem
// check: $(c_1=$VAL) = const u64 1
// check: $(index_inc=$VAL) = add $index, $c_1
// check: $(c_len=$VAL) = const u64 10
// check: $(continue=$VAL) = cmp lt $index_inc $c_len
// check: cbr $continue, array_init_loop($index_inc), array_init_loop_exit()

// The scalar `head` field (index 0) is stored in the exit block.
// check: array_init_loop_exit():
// check: $(c_head_idx=$VAL) = const u64 0
// check: $(ptr_head=$VAL) = get_elem_ptr $ptr_struct_init, __ptr u64, $c_head_idx
// check: $(c_3=$VAL) = const u64 3
// check: store $c_3 to $ptr_head
// check: load $ptr_struct_init

// There must be no `init_aggr` left after lowering.
// not: init_aggr
