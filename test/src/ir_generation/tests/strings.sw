script;

fn main() -> u64 {
    let s = "foo \t bar";
    f(s, s)
}

fn f(a: str[10], b: str[10]) -> u64 {
    // There are 2 strings.
    2
}

// This test is pretty broken, but only because our string support is pretty broken.  So this is
// really just snapshotting the current situation, but string support in the compiler needs to
// improve.

// check: local ptr string<10> s

// check: $(s_ptr=$VAL) = get_ptr ptr string<10> s, ptr string<10>, 0
// check: $(str_lit=$VAL) = const string<10> "foo \x5ct bar"
// check: store $str_lit, ptr $s_ptr

// check: $(s_ptr=$VAL) = get_ptr ptr string<10> s, ptr string<10>, 0
// check: $(lhs=$VAL) = load ptr $s_ptr
// check: $(s_ptr=$VAL) = get_ptr ptr string<10> s, ptr string<10>, 0
// check: $(rhs=$VAL) = load ptr $s_ptr
// check: $(res=$VAL) = call $ID($lhs, $rhs)
// check: ret u64 $res
