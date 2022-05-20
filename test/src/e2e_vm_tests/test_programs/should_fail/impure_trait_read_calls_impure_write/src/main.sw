contract;

trait A {
    #[storage(write)]
    fn f(self) -> bool;
}

impl A for bool {
    #[storage(write)]
    fn f(self) -> bool {
        self
    }
}

#[storage(read)]
pub fn g() -> bool {
    true.f()
}
