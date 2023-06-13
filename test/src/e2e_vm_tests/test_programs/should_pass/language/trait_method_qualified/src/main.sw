script;

struct S {}

impl S {
    fn method() -> u64 {
        1
    }
}

trait MySuperTrait {
    fn method() -> u64;
}

trait MyTrait: MySuperTrait {
    fn method() -> u64;
}

impl MySuperTrait for S {
    fn method() -> u64 {
        2
    }
}

impl MyTrait for S {
    fn method() -> u64 {
        3
    }
}

// same test case but the methods take `self` as a parameter
struct T {}

impl T {
    fn method_self(self) -> u64 {
        1
    }
}

trait MySuperTraitSelf {
    fn method_self(self) -> u64;
}

trait MyTraitSelf: MySuperTraitSelf {
    fn method_self(self) -> u64;
}

impl MySuperTraitSelf for T {
    fn method_self(self) -> u64 {
        2
    }
}

impl MyTraitSelf for T {
    fn method_self(self) -> u64 {
        3
    }
}

fn main() -> bool {
    assert(S::method() == 1);
    assert(<S as MySuperTrait>::method() == 2);
    assert(<S as MyTrait>::method() == 3);

    let t = T {};
    // t.method_self() disambiguates to T::method_self
    assert(t.method_self() == T::method_self(t));
    assert(<T as MySuperTraitSelf>::method_self(t) == 2);
    assert(<T as MyTraitSelf>::method_self(t) == 3);

    true
}
