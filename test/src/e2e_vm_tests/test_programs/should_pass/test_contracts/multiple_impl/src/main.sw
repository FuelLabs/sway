contract;

mod testlib;
mod testlib2;
use testlib2::bar;

abi TestContr {
    fn foo();
}

fn foo() {
    testlib::foo();
}

fn bar() {}

impl TestContr for Contract {
    fn foo() {
        foo();
        bar();
    }
}

fn main() {}
