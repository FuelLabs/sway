library lib;

pub struct MyStruct {
    x: u64, 
    y: bool
}

pub enum MyEnum {
    A: u64,
    B: bool,
}

impl core::ops::Eq for MyEnum {
    fn eq(self, other: MyEnum) -> bool {
        match (self, other) {
            (MyEnum::A(inner1), MyEnum::A(inner2))  => inner1 == inner2,
            (MyEnum::B(inner1), MyEnum::B(inner2))  => inner1 == inner2,
            _ => false,
        }
    }
}

configurable {
    C0: bool = true,
    C1: u64 = 42,
    C2: b256 = 0x1111111111111111111111111111111111111111111111111111111111111111,
    C3: MyStruct = MyStruct { x: 42, y: true },
    C4: MyEnum = MyEnum::A(42),
    C5: MyEnum = MyEnum::B(true),
    C6: str[4] = "fuel",
    C7: [u64; 4] = [1, 2, 3, 4],
}
