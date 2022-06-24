script;

fn foo<T>() {
    let x = __size_of::<T>();
}

fn main() {
    foo()
}
