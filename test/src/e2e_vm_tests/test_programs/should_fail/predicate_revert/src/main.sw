predicate;

fn main() -> bool {
    // This should produce a compile error: __revert is not allowed in predicates.
    // Predicates must evaluate to true/false, not abort execution.
    __revert(42);
    true
}
