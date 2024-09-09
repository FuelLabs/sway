script;

struct OneGeneric<U> {
    a: U,
}

struct TwoGenerics<T, V> {
    b: OneGeneric<T>,
    c: V,
}

type A = (TwoGenerics<u64, u32>, OneGeneric<u8>);

fn main(input: A) -> A {
    (
        TwoGenerics {
            b: OneGeneric { a: input.0.b.a + 1 },
            c: input.0.c + 1
        },
        OneGeneric { a: input.1.a + 1 },
    )
}
