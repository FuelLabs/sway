script;

fn main() {
    let _ = __add(1, 2);
    let _ = __sub(1, 2);
    let _ = __mul(1, 2);
    let _ = __div(1, 2);
    let _ = __and(1, 2);
    let _ = __or(1, 2);
    let _ = __xor(1, 2);
    let _ = __mod(1, 2);
    let _ = __rsh(1, 2);
    let _ = __lsh(1, 2);
}
// check: $(left=$VAL) = const u64 1, $MD
// check: $(right=$VAL) = const u64 2, $MD
// check: $VAL = add $left, $right

// check: $(left=$VAL) = const u64 1, $MD
// check: $(right=$VAL) = const u64 2, $MD
// check: $VAL = sub $left, $right

// check: $(left=$VAL) = const u64 1, $MD
// check: $(right=$VAL) = const u64 2, $MD
// check: $VAL = mul $left, $right

// check: $(left=$VAL) = const u64 1, $MD
// check: $(right=$VAL) = const u64 2, $MD
// check: $VAL = div $left, $right

// check: $(left=$VAL) = const u64 1, $MD
// check: $(right=$VAL) = const u64 2, $MD
// check: $VAL = and $left, $right

// check: $(left=$VAL) = const u64 1, $MD
// check: $(right=$VAL) = const u64 2, $MD
// check: $VAL = or $left, $right

// check: $(left=$VAL) = const u64 1, $MD
// check: $(right=$VAL) = const u64 2, $MD
// check: $VAL = xor $left, $right

// check: $(left=$VAL) = const u64 1, $MD
// check: $(right=$VAL) = const u64 2, $MD
// check: $VAL = mod $left, $right

// check: $(left=$VAL) = const u64 1, $MD
// check: $(right=$VAL) = const u64 2, $MD
// check: $VAL = rsh $left, $right

// check: $(left=$VAL) = const u64 1, $MD
// check: $(right=$VAL) = const u64 2, $MD
// check: $VAL = lsh $left, $right
