script;
fn main() {
    let null = asm(output) { zero: raw_ptr };
    let res = __state_load_quad(
      0x0000000000000000000000000000000000000000000000000000000000000001,
      null,
      1,
    );
}
// check: $(key=$VAL) = get_local __ptr b256, __anon_0, $(meta=$MD)
// check: $(ptr=$VAL) = cast_ptr $VAL to __ptr b256, $meta
// check: $(count=$VAL) = const u64 1, $MD
// check: $VAL = state_load_quad_word $ptr, key $key, $count, $meta
