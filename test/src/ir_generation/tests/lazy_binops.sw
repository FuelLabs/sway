script;

fn main() -> bool {
    (false && true) || true
}

// check: entry():
// check: $(false_val=$VAL) = const bool false
// check: cbr $false_val, $(bl0=$ID)(), $(bl1=$ID)($false_val)

// check: $bl0()
// check: $(bl0_val=$VAL) = const bool true
// check: br $bl1($bl0_val)

// check: $bl1($(bl1_val=$VAL): bool)
// check: cbr $bl1_val, $(bl3=$ID)($bl1_val), $(bl2=$ID)()

// check: $bl2()
// check: $(true_val=$VAL) = const bool true
// check: br $bl3

// check: $bl3($(ret_val=$VAL): bool)
// check: ret bool $ret_val
