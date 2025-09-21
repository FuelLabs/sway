script;

trait A {
    fn run() -> bool;
}

trait B {
    fn run() -> bool;
}

trait C {
    fn foo() -> u64;
}

trait D {
    fn bar() -> u64;
}

impl A for bool {
    fn run() -> bool {
        true
    }
}

impl B for bool {
    fn run() -> bool {
        true
    }
}

impl C for bool {
    fn foo() -> u64 {
        7
    }
}

impl D for bool {
    fn bar() -> u64 {
        11
    }
}

// check we select the trait from the bound when multiple traits implement the same method name
fn f<T>()
where
    T: A,
{
    if T::run() {
    }
}


// check disambiguation remains available
fn g() {
    if <bool as B>::run() {
        let _ = <bool as A>::run();
    }
}

// independent traits with distinct methods stay accessible
fn h() {
    let _ = <bool as C>::foo();
    let _ = <bool as D>::bar();
}

fn main() {
    f::<bool>();
    g();
    h();
}
