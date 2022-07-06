script;

fn main() -> u64 {
    if false {
        1_000_000
    } else {
        42
    }
}

// check: cbr $VAL, $(bl0=$ID), $(bl1=$ID)

// check: $bl0:
// check: $(bl0_val=$VAL) = const u64 1000000
// check: br $(bl2=$ID)

// check: $bl1:
// check: $(bl1_val=$VAL) = const u64 42
// check: br $bl2

// check: $bl2:
// check: $(ret_val=$VAL) = phi($bl0: $bl0_val, $bl1: $bl1_val)
// check: ret u64 $ret_val
