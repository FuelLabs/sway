script;

use trait_method_lib::*;

pub trait MyTrait2 {
    fn trait_method2(self) -> bool;
} {
    fn method2(self) -> MyStruct {
        MyStruct {}
    }
}

impl MyTrait for MyStruct {
    fn trait_method(self) -> bool {
        true
    }
}

impl MyTrait2 for MyStruct {
    fn trait_method2(self) -> bool {
        true
    }
}

fn main() {
    let s = MyStruct {};
    let b = s.trait_method();
    let b = s.trait_method2();
}
