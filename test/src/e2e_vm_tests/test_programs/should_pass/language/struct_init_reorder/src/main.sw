script;

pub struct MyInnerStruct {
    x: u64,
    y: u64,
}

pub struct MyStruct {
      value: MyInnerStruct,
}

pub enum MyEnum {
    V1: u8,
    V2: u64,
}

pub struct Foo {
    f1: MyEnum,
    f2: MyStruct,
}

fn main() {
    let f1 : MyEnum = MyEnum::V1(0u8);
    let f2 : MyStruct = MyStruct { value: MyInnerStruct { x: 0, y: 0 } };
    // f1 and f2 are instantiated in the wrong order below. that shouldn't matter.
    log(Foo {
        f2,
        f1
    });
}
