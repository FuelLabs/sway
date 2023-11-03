script;

trait TypeTrait {
    type T;

    fn method() -> Self::T;
} {
    fn method2<T>() -> T {
        Self::method()
    }
}

fn main() -> u32 {
    1
}