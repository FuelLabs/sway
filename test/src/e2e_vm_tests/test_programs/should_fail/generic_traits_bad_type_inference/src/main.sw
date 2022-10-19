script;

trait Setter<A> {
    fn set_it(self, new_value: A) -> Self;
}

struct FooBarData<B> {
    value: B
}

impl<C> Setter<C> for FooBarData<C> {
    fn set_it(self, new_value: C) -> Self {
        FooBarData {
            value: new_value
        }
    }
}

fn main() {
    let a = FooBarData {
        value: 1u8
    };
    let b = a.set_it(42);
    let c = a.set_it(false);
}
