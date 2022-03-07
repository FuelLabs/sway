predicate;

// In a script, there can be no impurity since storage is only available in contracts.
fn main() {
  foo()
}

impure fn foo() {
  
}
