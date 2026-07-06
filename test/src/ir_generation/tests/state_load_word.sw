script;

fn main() {
    let _ = __state_load_word(
      0x0000000000000000000000000000000000000000000000000000000000000001,
    );
}

// check: $(v0=$VAL) = get_local __ptr b256, __anon_0, $(meta=$MD)
// check: $(v1=$VAL) = const b256 0x0000000000000000000000000000000000000000000000000000000000000001,
// check: store $v1 to $v0
// check: $(v2=$VAL) = state_load_word key $v0, 0, $meta
// check: $(v3=$VAL) = get_local __ptr u64, _,
// check: store $v2 to $v3,
