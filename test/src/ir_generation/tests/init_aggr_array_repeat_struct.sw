script;

// A repeat array whose repeated element is itself an aggregate (a nested
// `init_aggr`). The nested aggregate is first initialized into its own
// temporary, then loaded once, and that loaded value is stored into every
// element of the array. The nested `init_aggr` is thus completely removed.

struct Record {
    a: u64,
    b: u64,
}

fn main() -> u64 {
    let a = [Record { a: 1, b: 2 }; 3];
    a[1].a
}

// ::check-ir::

// check: local [{ u64, u64 }; 3] __array_init_0
// check: local { u64, u64 } __struct_init_0

// The nested struct `init_aggr` provides the repeated value for the array `init_aggr`.
// check: $(ptr_struct_init=$VAL) = get_local __ptr { u64, u64 }, __struct_init_0
// check: $(nested_init_aggr=$VAL) = init_aggr $ptr_struct_init
// check: $(nested_load=$VAL) = load $nested_init_aggr
// check: init_aggr $(arr_ptr=$VAL) [$nested_load x 3]

// ::check-ir-optimized::
// pass: lower-init-aggr

// check: $(ptr_array_init=$VAL) = get_local __ptr [{ u64, u64 }; 3], __array_init_0
// check: $(ptr_struct_init=$VAL) = get_local __ptr { u64, u64 }, __struct_init_0

// The nested struct is initialized into its own temporary.
// check: $(c_0a=$VAL) = const u64 0
// check: $(ptr_field_a=$VAL) = get_elem_ptr $ptr_struct_init, __ptr u64, $c_0a
// check: $(c_1=$VAL) = const u64 1
// check: store $c_1 to $ptr_field_a
// check: $(c_1b=$VAL) = const u64 1
// check: $(ptr_field_b=$VAL) = get_elem_ptr $ptr_struct_init, __ptr u64, $c_1b
// check: $(c_2=$VAL) = const u64 2
// check: store $c_2 to $ptr_field_b

// The nested aggregate is loaded once and stored into every array element.
// check: $(repeated=$VAL) = load $ptr_struct_init
// check: $(idx_0=$VAL) = const u64 0
// check: $(ptr_elem_0=$VAL) = get_elem_ptr $ptr_array_init, __ptr { u64, u64 }, $idx_0
// check: store $repeated to $ptr_elem_0
// check: $(idx_1=$VAL) = const u64 1
// check: $(ptr_elem_1=$VAL) = get_elem_ptr $ptr_array_init, __ptr { u64, u64 }, $idx_1
// check: store $repeated to $ptr_elem_1
// check: $(idx_2=$VAL) = const u64 2
// check: $(ptr_elem_2=$VAL) = get_elem_ptr $ptr_array_init, __ptr { u64, u64 }, $idx_2
// check: store $repeated to $ptr_elem_2

// check: load $ptr_array_init

// There must be no `init_aggr` left after lowering.
// not: init_aggr
