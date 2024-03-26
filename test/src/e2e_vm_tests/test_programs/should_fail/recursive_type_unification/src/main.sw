script;

enum MyOption<T> {
    Some: T,
}

fn foo<T>(value: T) -> MyOption<T> {
    MyOption::Some(value)
}

fn bar<V>(value: V) -> MyOption<V> {
    MyOption::Some::<V>(foo::<V>(value))
}

fn main() {
    let x = bar(false);
}
