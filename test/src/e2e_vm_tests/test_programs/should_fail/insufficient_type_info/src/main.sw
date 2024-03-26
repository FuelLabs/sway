script;

mod lib;

use ::lib::MyOption::{self, *};

fn foo<T>() {
    let x = __size_of::<T>();
}

fn main() {
    foo();

    None::<T>;
}
