library;

/// A simple struct in library A
pub struct StructA {
    /// A field with documentation
    pub value: u64,
}

/// A trait in library A
pub trait TraitA {
    /// A method in the trait
    fn method_a(self) -> u64;
}

impl TraitA for StructA {
    /// Implementation of method_a for StructA
    fn method_a(self) -> u64 {
        self.value
    }
}

/// A public function in library A
pub fn function_a() -> u64 {
    42
}
