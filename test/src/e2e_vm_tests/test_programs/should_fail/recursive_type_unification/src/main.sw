script;

fn foo<T>(value: T) -> Option<T> {
    Some(value)
}

fn bar<V>(value: V) -> Option<V> {
    Some::<V>(foo::<V>(value))
}

fn main() {
    let x = bar(false);
}
