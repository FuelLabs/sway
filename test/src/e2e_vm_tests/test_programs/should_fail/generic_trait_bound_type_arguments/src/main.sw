script;

trait Mark<A> {}

struct NeedsU64<T>
where
    T: Mark<u64>,
{
    value: T,
}

struct OnlyU8 {
    n: u64,
}

impl Mark<u8> for OnlyU8 {}

fn forge<T>(x: T) -> NeedsU64<T>
where
    T: Mark<u8>,
{
    NeedsU64 { value: x }
}

fn main() -> u64 {
    let impossible: NeedsU64<OnlyU8> = forge(OnlyU8 { n: 42 });
    impossible.value.n
}

