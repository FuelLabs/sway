script;

struct GenericBimbam<U> {
    val: U,
}

struct GenericSnack<T, V> {
    twix: GenericBimbam<T>,
    mars: V,
}

fn main(baba: u64) -> u64 {
    __log((
        GenericSnack { twix: GenericBimbam { val: 1u64 }, mars: 2u32 },
        GenericBimbam { val: 3u8 },
    ));
    baba + 1
}
