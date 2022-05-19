contract;

trait A {
    #[storage(read)]
    fn f(self) -> bool;
}

impl A for bool {
    fn f(self) -> bool {
        self
    }
}
