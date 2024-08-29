script;

trait MyTrait{
    fn foo(ref mut self) -> u64;
}

impl MyTrait for u64 {
    fn foo(ref mut self) -> u64 {
        64
    }
}
impl MyTrait for u32 {
    fn foo(ref mut self) -> u64 {
        32
    }
}

impl u64 {
    fn baz(self) -> u64 {
        64
    }
}
impl u32 {
    fn baz(self) -> u64{
        32
    }
}

fn bar<T>(ref mut x: T) -> u64 where T: MyTrait {
    x.foo()
}

fn main() -> bool {
    let mut a = 0; // depth 1 type propagation
    let mut b = 0; // depth 2 type propagation
    let mut c = 0; // depth 3 type propagation
    assert_eq(bar(a), 32);
    assert_eq(bar(b), 32);
    assert_eq(bar(c), 32);
    {
        c = b;
    }
    {
        b = a;
    }
    {
        let _: &u32 = &a;
    }
    assert_eq(a.baz(), 32);
    assert_eq(b.baz(), 32);
    assert_eq(c.baz(), 32);
    {   // Test inside nested code block
        let mut a = 0; // depth 1 type propagation
        let mut b = 0; // depth 2 type propagation
        let mut c = 0; // depth 3 type propagation
        assert_eq(bar(a), 32);
        assert_eq(bar(b), 32);
        assert_eq(bar(c), 32);
        {
            c = b;
        }
        {
            b = a;
        }
        {
            let _: &u32 = &a;
        }
        assert_eq(a.baz(), 32);
        assert_eq(b.baz(), 32);
        assert_eq(c.baz(), 32);
    }

    true
}