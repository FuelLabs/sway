script;

mod concrete_types;
mod ref_and_ref_mut;
mod ref_and_ref_mut_trait_impls;

fn main() {
    ::concrete_types::test();
    ::ref_and_ref_mut::test();
}
