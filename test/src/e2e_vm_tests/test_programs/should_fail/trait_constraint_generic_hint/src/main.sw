script;

trait A {
    fn run() -> bool;
}

trait B {
    fn run() -> bool;
}

impl A for bool {
    fn run() -> bool {
        true
    }
}

impl B for bool {
    fn run() -> bool {
        false
    }
}

fn f<T>()
where
    T: A + B,
{
    if T::run() {
    }
}

fn main() {
    f::<bool>();
}
