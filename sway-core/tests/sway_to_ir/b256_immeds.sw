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
