script;

mod concrete_types;
mod ref_and_ref_mut;
mod ref_and_ref_mut_trait_impls;

fn main() -> u64 {
    assert_eq(::concrete_types::test(), 42);
    assert_eq(::ref_and_ref_mut::test(), 42);
    assert_eq(::ref_and_ref_mut_trait_impls::test(), 42);

    42
}
