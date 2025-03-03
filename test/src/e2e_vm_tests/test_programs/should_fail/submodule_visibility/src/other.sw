library;

mod lib; // private submodule

// lib is private, but since it is a direct submodule we can access its public items.
use lib::S;

// Public function
pub fn foo() {
    let my_struct = S { val: 0 };
}
