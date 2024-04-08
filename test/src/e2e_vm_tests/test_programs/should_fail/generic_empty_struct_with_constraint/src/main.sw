script;

trait Trait {}

// Does NOT have a Trait trait implementation
pub struct NoTraitStruct {
    val: u64
}

// Where clause on empty struct
pub struct GenericEmptyStruct<T> where T: Trait {}

fn main() {
    // Does not compile as expected
    let _: GenericEmptyStruct<NoTraitStruct> = GenericEmptyStruct {};
}