script;

fn main() -> u64 {
    if false {
        1_000_000
    } else {
        42
    }
}

// check: cbr $VAL, $(bl0=$ID)(), $(bl1=$ID)()

// check: $bl0():
// check: $(bl0_val=$VAL) = const u64 1000000
// check: br $(bl2=$ID)($bl0_val)

// check: $bl1():
// check: $(bl1_val=$VAL) = const u64 42
// check: br $bl2($bl1_val)

// check: $bl2($(ret_val=$VAL): u64):
// check: ret u64 $ret_val
