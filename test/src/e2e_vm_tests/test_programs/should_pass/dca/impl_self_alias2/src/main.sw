script;

struct MyStruct1 {}

type Alias1 = MyStruct1;

type Alias2 = Alias1;

impl Alias1 {
    fn foo() {}
}

fn main() {
    Alias2::foo();
}
