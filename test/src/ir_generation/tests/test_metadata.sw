script;

fn main() {}
fn my_func() {}

#[test()]
fn my_test_func() {
  my_func();
}

// check: fn main() -> (), $(main_md=$MD) {
// check: fn my_test_func() -> (), $(test_md=$MD) {

// check: $(decl_index_md=$MD) = decl_index
// check: $test_md = ($MD $decl_index_md)
