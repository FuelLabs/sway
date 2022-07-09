script;

fn main() -> bool {
    let a = 0x0202020202020202020202020202020202020202020202020202020202020202;
    cmp(a, 0x0303030303030303030303030303030303030303030303030303030303030303)
}

fn cmp(a: b256, b: b256) -> bool {
    asm(lhs: a, rhs: b, sz, res) {
        addi sz zero i32;
        meq res lhs rhs sz;
        res: bool
    }
}

// ::check-ir::

// check: local ptr b256 a
// check: $(ptr=$VAL) = get_ptr ptr b256 a, ptr b256, 0
// check: $(c1=$VAL) = const b256 0x0202020202020202020202020202020202020202020202020202020202020202
// check: store $c1, ptr $ptr

// check: const b256 0x0303030303030303030303030303030303030303030303030303030303030303
// check: $(res=$VAL) = call
// check: ret bool $res
