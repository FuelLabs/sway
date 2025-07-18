script;

fn main() -> u64 {
  let a = 1 + 2 + 3 + 4;
  a
}

// check: local u64 a
// check: $(v0=$VAL) = const u64 1
// check: $(v1=$VAL) = const u64 2
// check: $(v2=$VAL) = call add_0($v0, $v1)
// check: $(v3=$VAL) = const u64 3
// check: $(v4=$VAL) = call add_0($v2, $v3)
// check: $(v5=$VAL) = const u64 4
// check: $(v6=$VAL) = call add_0($v4, $v5)
// check: $(a_addr=$VAL) = get_local __ptr u64, a
// check: store $v6 to $a_addr

// ::check-ir-optimized::
// pass: o1

// not: local u64 a
// check: entry():
// check: $(a=$VAL) = const u64 10
// check: ret u64 $a
