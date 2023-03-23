script;

struct MyStruct1 {}

type Alias1 = MyStruct1;

fn main() {
    let _bar: Alias1 = MyStruct1 {};
}
