script;

enum MyOption<T> {
    Some: T,
    None: (),
}

fn main() -> bool {
    let _ = MyOption::None;
    true
}
