contract;

// this should fail because a pure function cannot call an impure function

fn main() {
}


fn pure_function() {
  impure_function();
}

impure fn impure_function() {}
