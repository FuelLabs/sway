script;

const SOME_TX_FIELD = 0x42;
const SOME_OTHER_TX_FIELD = 0x77;

fn main() {
    let field1 = __gtf::<u64>(1, SOME_TX_FIELD);
    let field2 = __gtf::<b256>(2, SOME_OTHER_TX_FIELD);
}

// check: local ptr u64 field1
// check: local ptr b256 field2

// check: $(gtf1_index=$VAL) = const u64 1
// check: $(gtf1=$VAL) = gtf $gtf1_index, 66
// check: $(field1_ptr=$VAL) = get_ptr ptr u64 field1, ptr u64, 0
// check:  store $gtf1, ptr $field1_ptr

// check: $(gtf2_index=$VAL) = const u64 2
// check: $(gtf2=$VAL) = gtf $gtf2_index, 119
// check: $(gtf2_int_to_ptr=$VAL) = int_to_ptr $gtf2 to b256, !6
// check: $(field2_ptr=$VAL) = get_ptr ptr b256 field2, ptr b256, 0
// check: store $gtf2_int_to_ptr, ptr $field2_ptr
