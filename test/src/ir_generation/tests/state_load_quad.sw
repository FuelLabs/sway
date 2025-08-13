script;
fn main() {
    let null = __transmute::<u64, raw_ptr>(0);
    let res = __state_load_quad(
      0x0000000000000000000000000000000000000000000000000000000000000001,
      null,
      1,
    );
}
// check: $(key=$VAL) = get_local __ptr b256, key_for_storage, $(meta=$MD)
// check: $(ptr=$VAL) = cast_ptr $VAL to __ptr b256, $meta
// check: $(count=$VAL) = const u64 1, $MD
// check: $VAL = state_load_quad_word $ptr, key $key, $count, $meta
