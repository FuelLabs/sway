script;

use trait_method_lib::*;

impl MyTrait for MyStruct {
    fn trait_method(self) -> bool {
        true
    }
}

fn main() {
    let s = MyStruct {};
    let b = s.trait_method();
}
