script;

trait A {
    fn associated_method() -> bool;
    fn method(self) -> bool;
}

trait B {
    fn associated_method() -> bool;
    fn method(self) -> bool;
}

impl A for bool {
    #[allow(dead_code)]
    fn associated_method() -> bool {
        true
    }

    fn method(self) -> bool {
        true
    }
}

impl B for bool {
    #[allow(dead_code)]
    fn associated_method() -> bool {
        false
    }

    fn method(self) -> bool {
        false
    }
}

#[allow(dead_code)]
fn f<T>(item: T)
where
    T: A + B,
{
    if T::associated_method() {
    }

    if item.method() {

    }
}

fn main() {
    f::<bool>(true);
}
