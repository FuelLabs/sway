script;

trait Mark<A> {}

struct NeedsU64<T>
where
    T: Mark<u64>,
{
    value: T,
}

struct HasU64 {
    n: u64,
}

impl Mark<u64> for HasU64 {}

fn build<T>(x: T) -> NeedsU64<T>
where
    T: Mark<u64>,
{
    NeedsU64 { value: x }
}

fn main() -> u64 {
    let valid: NeedsU64<HasU64> = build(HasU64 { n: 42 });
    valid.value.n
}

