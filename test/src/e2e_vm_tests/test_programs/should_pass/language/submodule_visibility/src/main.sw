library;

// other is a submodule that declares a private submodule lib
// lib contains a declaration of the public struct S, but since lib is private it is not visible here.
// It is visible inside other, though.
pub mod other;

