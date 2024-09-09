script;

struct MyStruct<T> {
    val: T
}

trait MyTrait {
    fn foo(self, other: Self) -> bool;
} {
    fn bar(self, other: Self) -> bool {
        self.foo(other)
    }
}

impl <T> MyTrait for MyStruct<T> where T: MyTrait {
    fn foo(self, other: Self) -> bool {
        self.val.foo(other.val)
    }
}

fn main() -> bool {
    let my_struct_1 = MyStruct { val: 5 };
    let my_struct_2 = MyStruct { val: 9 };

    // Calling foo() gives us the following expected error:
    // Trait "MyTrait" is not implemented for type "u64".
    let _ = my_struct_1.foo(my_struct_2);

    // Calling bar() gives us the following expected error:
    // Trait "MyTrait" is not implemented for type "u64".
    let _ = my_struct_1.bar(my_struct_2);

    true
}