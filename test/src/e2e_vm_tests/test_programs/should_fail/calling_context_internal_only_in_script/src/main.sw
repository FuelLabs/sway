script;

#[context(internal_only)]
fn bar() {}

#[context(internal_only)]
fn baz() {
  bar();
}

#[context(internal_only)]
fn main() {
  baz();
}
