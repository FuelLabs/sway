contract;

mod foo;

/// This doc comment
/// should return a parser error
mod bar;

#[inline(never)]
pub mod baz;

fn a() -> bool {
    0 // Test that recovery reaches type checking.
}
