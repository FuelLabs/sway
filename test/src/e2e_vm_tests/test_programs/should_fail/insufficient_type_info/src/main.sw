script;

fn foo<T>() {
    let x = size_of::<T>();
}

fn main() {
    foo()
}
