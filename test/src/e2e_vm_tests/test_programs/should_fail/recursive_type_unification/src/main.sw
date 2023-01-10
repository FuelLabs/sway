script;

fn foo<T>(value: T) -> Option<T> {
    Option::Some(value)
}

fn bar<V>(value: V) -> Option<V> {
    Option::Some::<V>(foo::<V>(value))
}

fn main() {
    let x = bar(false);
}
