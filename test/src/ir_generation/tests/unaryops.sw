script;

fn main() {
    let _ = __not(0u8);
    let _ = __not(0u16);
    let _ = __not(0u32);
    let _ = __not(0u64);
}

// check: $(a=$VAL) = const u8 0, $MD
// check: $VAL = not $a

// check: $(b=$VAL) = const u64 0, $MD
// check: $VAL = not $b

// check: $(c=$VAL) = const u64 0, $MD
// check: $VAL = not $c

// check: $(d=$VAL) = const u64 0, $MD
// check: $VAL = not $d
