library;

use lib_a::*;

/// A struct in library B that uses library A
pub struct StructB {
    /// Contains a StructA
    pub inner: StructA,
}

/// A trait in library B
pub trait TraitB {
    /// A method that returns a string
    fn method_b(self) -> str;
}

impl TraitB for StructB {
    /// Implementation of method_b for StructB
    fn method_b(self) -> str {
        "hello"
    }
}

/// A public function in library B that uses library A
pub fn function_b() -> u64 {
    function_a() + 1
}
