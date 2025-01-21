library;

mod lib; // private submodule

// lib is private, but since it is a direct submodule we can access its public items.
use lib::S;

// lib is private, but since it is a direct submodule we can access its public items.
// Reexporting it makes it visible from here, but not from lib
pub use lib::U;

// Public function
pub fn foo() {
    let my_struct = S { val: 0 };

    // lib is private, but since it is a direct submodule we can access its public items.
    let my_other_struct = lib::T { val: 1 };
}
