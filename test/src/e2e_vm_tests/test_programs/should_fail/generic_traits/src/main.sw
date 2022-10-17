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

fn main() -> u8 {
    let foo = FooBarData {
        value: 1u8
    };
    let bar = foo.set(42);
    bar.value
}
