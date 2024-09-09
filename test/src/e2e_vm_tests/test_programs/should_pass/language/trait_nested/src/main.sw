script;

trait T1 {
    fn trait_fn() -> Self;
}

impl T1 for u64 {
    fn trait_fn() -> u64 {
        42
    }
}

impl<A> T1 for (A, ) 
where
    A: T1
{
    fn trait_fn() -> (A, ) {
        (A::trait_fn(), )
    }
}

fn f<T> () -> T
where
    T: T1
{
    T::trait_fn()
}

impl Eq for (u64,) {
    fn eq(self, other: Self) -> bool {
        self.0 == other.0
    }
}

fn main() -> bool {
    assert_eq(f::<(u64, )>(), (42,));

    true
}