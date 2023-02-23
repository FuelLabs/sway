contract;

trait Trait1 {}
trait Trait2 {}

fn fun<A, B>(a: A) where
    A: Trait1,
    B: Trait1 + Trait2
{}

