script;

trait Setter<A> {
    fn set(self, new_value: A) -> Self;
}

struct FooBarData<B> {
    value: B
}

impl<C> Setter<C> for FooBarData<C> {
    fn set(self, new_value: C) -> Self {
        FooBarData {
            value: new_value,
        }
    }
}

trait Returner<D> {
    fn return_it(self, the_value: D) -> D;
}

impl<E, F> Returner<E> for FooBarData<F> {
    fn return_it(self, the_value: E) -> E {
        the_value
    }
}

fn main() -> u64 {
    let a = FooBarData {
        value: 1u8
    };
    let b = a.set(42);
    let c = b.value;
    let d = b.return_it(true);
    let e = b.return_it(9u64);

    if c == 42u8 && d && e == 9u64 {
        42
    } else {
        7
    }
}
