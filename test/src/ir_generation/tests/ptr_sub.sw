script;

fn main() {
    let null = __transmute::<u64, raw_ptr>(0);
    let _ = __ptr_sub::<u64>(null, 1);
}
// check: $VAL = get_local __ptr ptr, null, $MD
// check: $(ptr=$VAL) = get_local __ptr ptr, null, $MD
// check: $(ptr_op=$VAL) = load $ptr
// check: $(size=$VAL) = const u64 8
// check: $(count=$VAL) = const u64 1, $MD
// check: $(mul_res=$VAL) = mul $size, $count
// check: $(add_res=$VAL) = sub $ptr_op, $mul_res
// check: $(dst_ptr=$VAL) = get_local __ptr ptr, _, $(dst_md=$MD)
// check: store $add_res to $dst_ptr, $dst_md
