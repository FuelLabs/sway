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

trait MyTrait : MySuperTrait {
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

fn main() -> bool {
    assert(S::method() == 1);
    assert(<S as MySuperTrait>::method() == 2);
    assert(<S as MyTrait>::method() == 3);

    true   
}