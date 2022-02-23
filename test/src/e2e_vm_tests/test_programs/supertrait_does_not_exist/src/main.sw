script;

trait A {
    fn a();
}

// Trait C does not exist in this scop. This shouldn't compile
trait B: C {
    fn b();
}

fn main() { } 
