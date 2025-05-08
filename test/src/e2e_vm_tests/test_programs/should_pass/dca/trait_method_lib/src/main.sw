library;

pub struct MyStruct {

}

pub trait MyTrait {
    fn trait_method(self) -> bool;
} {
    fn method(self) -> MyStruct {
        MyStruct {}
    }
}

impl MyTrait for MyStruct {
    fn trait_method(self) -> bool {
        true
    }
}
