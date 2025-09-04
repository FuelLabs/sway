library;

pub struct MyStruct { }

pub enum MyEnum {
    A: ()
}

trait MyTrait {}

impl MyTrait for MyStruct {}
impl MyTrait for MyEnum {}
