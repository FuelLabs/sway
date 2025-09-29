script;

use std::constants::ZERO_B256;

struct MyStruct<T> {
    val: u64
}

trait From<T> {
    fn from2(num: T) -> Self;
    fn try_from2(num: T) -> Option<Self>;
    fn into2(self) -> T;
}

// Implements on MyStruct<T>
impl<T> From<b256> for MyStruct<T> {
    fn from2(num: b256) -> Self {
        MyStruct { val: 0 }
    }

    fn try_from2(num: b256) -> Option<Self> {
        Some(MyStruct { val: 0 })
    }

    fn into2(self) -> b256 {
        ZERO_B256
    }
}

// Implements on MyStruct<u64>
impl From<u64> for MyStruct<u64> {
    fn from2(num: u64) -> Self {
        MyStruct { val: 0 }
    }

    fn try_from2(num: u64) -> Option<Self> {
        Some(MyStruct { val: 0 })
    }

    fn into2(self) -> u64 {
        0
    }
}

// https://github.com/FuelLabs/sway/issues/7398

trait A { fn f() -> bool; }

impl A for u64 { fn f () -> bool { true } }
impl A for bool { fn f () -> bool { false } }

fn ff<T>() -> bool where T: A {
    let v: bool = T::f();
    v
}

fn main() -> bool {
    let my_struct: MyStruct<u64> = MyStruct { val: 1 };
    let my_b256 = ZERO_B256;
    let my_u64 = 1_u64;

    let _into_b256: b256 = my_struct.into2();
    let _into_u64: u64 = my_struct.into2();

    let _from_b256: MyStruct<b256> = MyStruct::from2(my_b256);
    let _from_u64: MyStruct<u64> = MyStruct::from2(my_u64);

    let _try_from_b256: Option<MyStruct<b256>> = MyStruct::try_from2(my_b256);
    let _try_from_u64: Option<MyStruct<u64>> = MyStruct::try_from2(my_u64);

    // https://github.com/FuelLabs/sway/issues/7398
    if !(ff::<u64>()) {
        __revert(0);
    }

    true
}