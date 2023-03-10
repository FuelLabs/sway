script;

#[allow(dead_code)]
struct MyStruct1 {}

#[allow(dead_code)]
type Alias1 = MyStruct1;

#[allow(dead_code)]
type Alias2 = MyStruct1;

impl Alias1 {
    fn foo() {}
}

impl Alias2 {
    fn foo() {}
}

fn main() {
}
