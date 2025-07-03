script;

fn main() {
    let null = asm(output) { zero: raw_ptr };
    let _ = __ptr_add::<u64>(null, 1);
}
// check: $VAL = get_local __ptr u64, null, $MD
// check: $(ptr=$VAL) = get_local __ptr u64, null, $MD
// check: $(ptr_op=$VAL) = load $ptr
// check: $(size=$VAL) = const u64 8
// check: $(count=$VAL) = const u64 1, $MD
// check: $(mul_res=$VAL) = mul $size, $count
// check: $(add_res=$VAL) = add $ptr_op, $mul_res
// check: $(dst_ptr=$VAL) = get_local __ptr u64, _, $(dst_md=$MD)
// check: store $add_res to $dst_ptr, $dst_md
