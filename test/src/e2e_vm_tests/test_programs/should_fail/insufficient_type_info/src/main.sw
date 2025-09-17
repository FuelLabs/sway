library;

mod lib;

use ::lib::MyOption::{self, *};

fn foo<T>() {
    let x = __size_of::<T>();
}

pub fn main() {
    foo();

    None::<T>;
}
