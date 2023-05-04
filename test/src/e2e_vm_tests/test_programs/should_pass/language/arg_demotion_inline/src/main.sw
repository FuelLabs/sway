contract;

abi MyContract {
    fn test_function() -> bool;
}

struct Foo {
    a: u64,
    b: u64,
    c: u64,
    d: u64,
    e: u64,
    f: u64,
}

impl Foo {
    fn new(
        a: u64,
        b: u64,
        c: u64,
        d: u64,
        e: u64,
        f: u64,
    ) -> Self {
        Self {
            a,
            b,
            c,
            d,
            e,
            f,
        }
    }
}

impl MyContract for Contract {
    fn test_function() -> bool {
        let bar1 = Foo::new(0,0,0,0,0,0);
        let bar2 = Foo::new(0,0,0,0,0,0);
        true
    }
}
