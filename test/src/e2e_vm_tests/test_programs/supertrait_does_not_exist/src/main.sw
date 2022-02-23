script;

trait A {
    fn a();
}

// Trait C does not exist in this scope. This shouldn't compile
trait B: C {
    fn b();
}

fn main() { } 
