script;

fn main() -> bool {
    (false && true) || true
}

// check: entry:
// check: $(false_val=$VAL) = const bool false
// check: cbr $false_val, $(bl0=$ID), $(bl1=$ID)

// check: $bl0:
// check: $VAL = phi(entry: $false_val)
// check: $(bl0_val=$VAL) = const bool true
// check: br $bl1

// check: $bl1:
// check: $(bl1_val=$VAL) = phi(entry: $false_val, $bl0: $bl0_val)
// check: cbr $bl1_val, $(bl3=$ID), $(bl2=$ID)

// check: $bl2:
// check: $VAL = phi($bl1: $bl1_val)
// check: $(true_val=$VAL) = const bool true
// check: br $bl3

// check: $bl3:
// check: $(ret_val=$VAL) = phi($bl1: $bl1_val, $bl2: $true_val)
// check: ret bool $ret_val
