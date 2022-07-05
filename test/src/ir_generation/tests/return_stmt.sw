script;

fn go(test: bool) -> u64 {
    if test {
        return 0;
    };
    if test {
        1
    } else {
        2
    }
}

fn main() -> u64 {
    go(true)
}

// check: cbr $ID, $(block0=$ID), $(block1=$ID)

// check: $block0:
// check: $(zero_val=$VAL) = const u64 0
// check: ret u64 $zero_val

// check: $block1:
// check: $(unit_val=$VAL) = const unit ()
// check: br $(block2=$ID)

// check: $block2:
// check: $VAL = phi($block1: $unit_val)
