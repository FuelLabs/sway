script;

fn main() {
    let _ = __state_store_word(
      0x0000000000000000000000000000000000000000000000000000000000000001,
      1,
    );
}
// check: $(key=$VAL) = get_local __ptr b256, __anon_0, $(meta=$MD)
// check: $(count=$VAL) = const u64 1, $MD
// check: $VAL = state_store_word $count, key $key, $meta
