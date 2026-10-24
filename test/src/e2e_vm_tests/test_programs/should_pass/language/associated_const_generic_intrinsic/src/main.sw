script;

struct Wrapper<T> {}

impl<T> Wrapper<T> {
    const IS_ZERO_SIZED: bool = __size_of::<T>() == 0;
}

fn main() {
    let _ = Wrapper::<u64> {};
}
