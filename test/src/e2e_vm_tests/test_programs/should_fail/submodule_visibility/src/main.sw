library;

// other is a submodule that declares a private submodule lib
// lib contains a declaration of the public struct S, but since lib is private it is not visible here.
// It is visible inside other, though.
pub mod other;

// lib is private, and not a direct submodule of the current module, so this should fail
use other::lib::S; 

pub fn foo() {
    let my_struct = S { val: 0 };
}
