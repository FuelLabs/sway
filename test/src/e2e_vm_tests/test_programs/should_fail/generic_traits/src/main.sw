script;

dep helpers;

trait Setter<A> {
    fn set(self, new_value: A) -> Self;
}

trait Getter<B> {
    fn get(self) -> B;
}

trait Returner<C> {
    fn return_it(self, the_value: C) -> C;
}

struct FooBarData<D> {
    value: D
}

// F is unconstrained
impl<E, F> Setter<E> for FooBarData<E> {
    fn set(self, new_value: E) -> Self {
        FooBarData {
            value: new_value,
        }
    }
}

impl<G, H> Returner<G> for FooBarData<H> {
    fn return_it(self, the_value: G) -> G {
        the_value
    }
}

// OutOfScopeGetter is not in this scope
impl<I> OutOfScopeGetter<I> for FooBarData<I> {
    fn out_of_scope_get(self) -> I {
        self.value
    }
}

// Getter only takes 1 type argument, not 2
impl<J, K> Getter<J, K> for FooBarData<J> {
    fn get(self) -> J {
        self.value
    }
}

// Getter must take a type argument
impl<L> Getter for FooBarData<L> {
    fn get(self) -> L {
        self.value
    }
}

impl<M> Getter<M> for FooBarData<M> {
    fn get(self) -> M {
        self.value
    }
}

trait Unused<N> {
    fn unused(self, x: u64, other: N) -> u64;
}

trait Multiple<T> {
    fn unused(self, x: u64, other: T) -> u64;
}

impl<T> Multiple<u64> for FooBarData<T> {
    fn unused(self, x: u64, other: u64) -> u64 {
        other
    }
}

// Conflicting definitions
impl<F> Multiple<u64> for FooBarData<F> {
    fn unused(self, x: u64, other: u64) -> u64 {
        other
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
    let f = b.get();

    if c == 42u8 && d && e == 9u64 && f == 42 {
        42
    } else {
        7
    }
}
