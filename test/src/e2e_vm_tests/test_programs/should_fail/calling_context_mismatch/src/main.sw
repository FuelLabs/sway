script;

#[context(internal_only)]
fn bar() {}

fn baz() {
  bar();
}

fn main() {
  baz();
}
