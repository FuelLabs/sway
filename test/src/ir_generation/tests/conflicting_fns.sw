script;

struct A {
    a: u64,
}

impl A {
    fn foo(self, val: u64) -> u64 {
        asm(a: val, res) {
            add res a a;
            res: u64
        }
    }
}

struct M {
    m: u64,
}

impl M {
    fn foo(self, val: u64) -> u64 {
        asm(a: val, res) {
            mul res a a;
            res: u64
        }
    }
}

fn main() -> u64 {
    let a = (A { a: 0 }).foo(10);        // 10 + 10 = 20
    let m = (M { m: 0 }).foo(4);         // 4 * 4 = 16
    asm (l: a, r: m, res) {
        sub res l r;                    // 20 - 16 = 4
        res: u64
    }
}

// This tests make sure that functions with the same name in the same
// module end up having an unique name when IR generated.
// Reference: https://github.com/FuelLabs/sway/pull/2330#discussion_r921809763

// ::check-ir::
// check: fn foo_0(
// check: fn foo_1(