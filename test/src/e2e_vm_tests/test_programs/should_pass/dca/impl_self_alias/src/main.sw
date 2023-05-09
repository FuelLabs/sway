script;

struct MyStruct1 {}

type Alias1 = MyStruct1;

type Alias2 = MyStruct1;

impl Alias1 {
    fn foo() {}
}

fn main() {
    Alias1::foo();
}
