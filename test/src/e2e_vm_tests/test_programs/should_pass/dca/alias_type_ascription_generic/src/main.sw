script;

struct MyStruct1 {}

struct MyStruct2<T> {}

type Alias1 = MyStruct1;

fn main() {
    let _bar: MyStruct2<Alias1> = MyStruct2 {};
}
