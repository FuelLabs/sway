contract;

dep foo;

/// This doc comment
/// should return a parser error
dep bar;

#[inline(never)]
pub dep baz;

fn a() -> bool {
    0 // Test that recovery reaches type checking.
}
