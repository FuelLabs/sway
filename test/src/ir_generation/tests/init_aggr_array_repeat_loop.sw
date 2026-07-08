script;

// Large repeat arrays (length > 5) are lowered into an initialization loop.
fn main() -> u64 {
    let a = [7u64; 10];
    a[1]
}

// ::check-ir::

// check: local [u64; 10] __array_init_0

// check: $(ptr_array_init=$VAL) = get_local __ptr [u64; 10], __array_init_0
// check: $(c_7=$VAL) = const u64 7
// check: init_aggr $ptr_array_init [$c_7 x 10]

// ::check-ir-optimized::
// pass: lower-init-aggr

// The entry block branches into the loop with index 0.
// check: $(ptr_array_init=$VAL) = get_local __ptr [u64; 10], __array_init_0
// check: $(idx_init=$VAL) = const u64 0
// check: br array_init_loop($idx_init)

// The loop block: store the repeated value into `array[index]`.
// check: array_init_loop(mut $(index=$VAL): u64):
// check: $(ptr_elem=$VAL) = get_elem_ptr $ptr_array_init, __ptr u64, $index
// check: $(c_7=$VAL) = const u64 7
// check: store $c_7 to $ptr_elem

// Increment the index and compare against the length.
// check: $(c_1=$VAL) = const u64 1
// check: $(index_inc=$VAL) = add $index, $c_1
// check: $(c_len=$VAL) = const u64 10
// check: $(continue=$VAL) = cmp lt $index_inc $c_len
// check: cbr $continue, array_init_loop($index_inc), array_init_loop_exit()

// The exit block loads the fully initialized array.
// check: array_init_loop_exit():
// check: load $ptr_array_init

// There must be no `init_aggr` left after lowering.
// not: init_aggr
