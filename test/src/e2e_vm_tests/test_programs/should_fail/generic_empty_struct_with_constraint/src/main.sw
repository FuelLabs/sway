library;

trait Trait {}

// Does NOT have a Trait trait implementation
pub struct NoTraitStruct {}

// Where clause on empty struct
pub struct GenericEmptyStruct<T> where T: Trait {}

pub fn main() {
    // Does not compile as expected
    let _: GenericEmptyStruct<NoTraitStruct> = GenericEmptyStruct {};
}