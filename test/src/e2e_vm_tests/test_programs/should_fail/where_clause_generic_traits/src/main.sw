library;

struct A {}

trait MyTraitGeneric<T> {
    fn method(self) -> u64;
}

impl MyTraitGeneric<u64> for u64 {
    fn method(self) -> u64 {
        1
    }
}

/* Missing impl
impl MyTraitGeneric<A> for u64 {
    fn method(self) -> u64 {
        2
    }
}
*/

impl MyTraitGeneric<u64> for u32 {
    fn method(self) -> u64 {
        1
    }
}

/* Missing impl
impl MyTraitGeneric<A> for u32 {
    fn method(self) -> u64 {
        2
    }
}
*/

trait MyTraitGeneric2<T> where T: MyTraitGeneric<u64> + MyTraitGeneric<A> {
    fn method2(self) -> u64;
}

impl MyTraitGeneric2<u64> for u64 {
    fn method2(self) -> u64 {
        <u64 as MyTraitGeneric<A>>::method(self) + <u64 as MyTraitGeneric<u64>>::method(self)
    }
}

impl MyTraitGeneric2<u32> for u32 {
    fn method2(self) -> u64 {
        <u32 as MyTraitGeneric<A>>::method(self) + <u32 as MyTraitGeneric<u64>>::method(self)
    }
}

pub trait MyTraitGeneric3<T> {
    fn method3(self) -> u64;
}

impl<T> MyTraitGeneric3<T> for T
where
    T: MyTraitGeneric2<T>,
{
    fn method3(self) -> u64 {
        T::method2(self)
    }
}