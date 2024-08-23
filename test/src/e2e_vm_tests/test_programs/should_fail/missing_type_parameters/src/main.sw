script;

fn main() {
    let g: bool = three_generics(true, "foo", 10);
    
    // Should fail because compiler cannot infer generic argument
    one_generic();
}

fn three_generics(a: A, b: B, c: C) -> A {
    let new_a: A = a;
    new_a
}

fn one_generic<T>() {

}
